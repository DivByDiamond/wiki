"""Deterministic orthographic JSON-model rasterizer; Pillow + NumPy only."""
import io,json,math,urllib.request
import numpy as np
from PIL import Image

class Assets:
    def __init__(self, root, jar):
        self.root,self.jar=root,jar
        self.downloads={};self.urls=[]
    def read(self, namespace, path):
        if namespace=='oc2r': return (self.root/'assets'/namespace/path).read_bytes()
        if path.startswith('textures/'):
            url='https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.1/assets/'+namespace+'/'+path
            if url not in self.downloads:
                self.downloads[url]=urllib.request.urlopen(url,timeout=40).read();self.urls.append(url)
                assert self.downloads[url]==self.jar.read('assets/'+namespace+'/'+path),path
            return self.downloads[url]
        return self.jar.read('assets/'+namespace+'/'+path)
    def model(self, name):
        ns,path=name.split(':') if ':' in name else ('minecraft',name)
        if path.startswith('builtin/'): return {}
        data=json.loads(self.read(ns,'models/'+path+'.json'))
        base=self.model(data['parent']) if 'parent' in data else {}
        for key in ('textures','display'):
            data[key]={**base.get(key,{}),**data.get(key,{})}
        return {**base,**data}
    def texture(self,name):
        ns,path=name.split(':') if ':' in name else ('minecraft',name)
        im=Image.open(io.BytesIO(self.read(ns,'textures/'+path+'.png'))).convert('RGBA')
        if im.height>im.width: im=im.crop((0,0,im.width,im.width))
        return im

def rotation(angles):
    x,y,z=np.radians(angles)
    a=np.array([[1,0,0],[0,np.cos(x),-np.sin(x)],[0,np.sin(x),np.cos(x)]])
    b=np.array([[np.cos(y),0,np.sin(y)],[0,1,0],[-np.sin(y),0,np.cos(y)]])
    c=np.array([[np.cos(z),-np.sin(z),0],[np.sin(z),np.cos(z),0],[0,0,1]])
    return c@a@b

def face_points(lo,hi,side):
    x,y,z=lo; X,Y,Z=hi
    return {'north':[(X,Y,z),(x,Y,z),(x,y,z),(X,y,z)],
      'south':[(x,Y,Z),(X,Y,Z),(X,y,Z),(x,y,Z)],
      'east':[(X,Y,Z),(X,Y,z),(X,y,z),(X,y,Z)],
      'west':[(x,Y,z),(x,Y,Z),(x,y,Z),(x,y,z)],
      'up':[(x,Y,z),(X,Y,z),(X,Y,Z),(x,Y,Z)],
      'down':[(x,y,Z),(X,y,Z),(X,y,z),(x,y,z)]}[side]

def default_uv(lo,hi,side):
    x,y,z=lo; X,Y,Z=hi
    return {'north':[16-X,16-Y,16-x,16-y], 'south':[x,16-Y,X,16-y],
     'west':[z,16-Y,Z,16-y], 'east':[16-Z,16-Y,16-z,16-y],
     'up':[x,z,X,Z], 'down':[x,16-Z,X,16-z]}[side]

def render_model(model,assets):
    textures=model.get('textures',{})
    def texture(key):
        while key.startswith('#') or key in textures:key=textures[key[1:] if key.startswith('#') else key]
        return np.asarray(assets.texture(key))
    gui=model.get('display',{}).get('gui',{})
    matrix=rotation(gui.get('rotation',[30,225,0]))
    scale=np.array(gui.get('scale',[.625]*3))
    shift=np.array(gui.get('translation',[0,0,0]))
    quads=[]
    for elem in model.get('elements',[]):
        lo,hi=elem['from'],elem['to']
        for side,face in elem.get('faces',{}).items():
            pts=np.array(face_points(lo,hi,side),dtype=float)
            if 'rotation' in elem:
                r=elem['rotation'];origin=np.array(r['origin']);angles=[0,0,0]
                angles['xyz'.index(r['axis'])]=r['angle']
                pts=(pts-origin)@rotation(angles).T+origin
                if r.get('rescale'):
                    # Minecraft scales the axes perpendicular to the rotation axis.
                    factor=np.ones(3)/math.cos(math.radians(r['angle']))
                    factor['xyz'.index(r['axis'])]=1
                    pts=(pts-origin)*factor+origin
            pts=((pts-8)*scale)@matrix.T+shift
            normal=np.cross(pts[1]-pts[0],pts[2]-pts[0])
            if normal[2]>=-1e-8:continue
            uv=face.get('uv',default_uv(lo,hi,side));u,v,U,V=uv
            coords=np.array([[u,v],[U,v],[U,V],[u,V]],float)
            coords=np.roll(coords,-face.get('rotation',0)//90,axis=0)
            brightness={'up':1,'down':.5,'north':.8,'south':.8,'east':.65,'west':.65}[side]
            quads.append((pts,coords,texture(face['texture']),brightness))
    size=128;rgba=np.zeros((size,size,4),dtype=np.uint8);depth=np.full((size,size),-np.inf)
    for pts,uv,tex,shade in quads:
        screen=np.stack([64+pts[:,0]*8,64-pts[:,1]*8],axis=1)
        for indices in ((0,1,2),(0,2,3)):
            ids=list(indices);p=screen[ids];q=pts[ids];t=uv[ids]
            left,top=np.maximum(np.floor(p.min(0)).astype(int),0)
            right,bottom=np.minimum(np.ceil(p.max(0)).astype(int),size-1)
            if left>right or top>bottom:continue
            xx,yy=np.meshgrid(np.arange(left,right+1)+.5,np.arange(top,bottom+1)+.5)
            bary=np.stack([xx-p[2,0],yy-p[2,1]],axis=-1)
            mat=np.stack([p[0]-p[2],p[1]-p[2]],axis=1)
            if abs(np.linalg.det(mat))<1e-9:continue
            ab=bary@np.linalg.inv(mat).T
            weights=np.concatenate([ab,1-ab.sum(-1,keepdims=True)],axis=-1)
            z=weights@q[:,2];uvs=weights@t
            tx=np.clip((uvs[:,:,0]/16*tex.shape[1]).astype(int),0,tex.shape[1]-1)
            ty=np.clip((uvs[:,:,1]/16*tex.shape[0]).astype(int),0,tex.shape[0]-1)
            colors=tex[ty,tx].copy();colors[:,:,:3]=(colors[:,:,:3]*shade).astype('uint8')
            target=rgba[top:bottom+1,left:right+1];zbuffer=depth[top:bottom+1,left:right+1]
            mask=(weights.min(-1)>=-1e-6)&(z>=zbuffer)&(colors[:,:,3]>0)
            target[mask]=colors[mask];zbuffer[mask]=z[mask]
    return Image.fromarray(rgba)

def robot_model():
    elements=[]
    for lo,dim,offset in [([-7,8,-7],[14,6,14],[1,1]),([-7,0,-7],[14,7,14],[1,23]),([-6,7,-6],[12,1,12],[1,34])]:
        x,y,z=lo;w,h,d=dim;u,v=offset
        # Robot renderer keeps ModelPart Y upright (unlike vanilla living renderers).
        faces={'west':[u,v+d,u+d,v+d+h],'north':[u+d,v+d,u+d+w,v+d+h],
          'east':[u+d+w,v+d,u+2*d+w,v+d+h], 'south':[u+2*d+w,v+d,u+2*d+2*w,v+d+h],
          'down':[u+d,v,u+d+w,v+d], 'up':[u+d+w,v,u+d+2*w,v+d]}
        elements.append({'from':[x+8,y,z+8],'to':[x+w+8,y+h,z+d+8],
          'faces':{s:{'texture':'#robot','uv':[a/4 for a in uv]} for s,uv in faces.items()}})
    return {'textures':{'robot':'oc2r:entity/robot/robot'},'elements':elements,
      'display':{'gui':{'rotation':[30,225,0],'scale':[.75]*3}}}
