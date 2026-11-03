"""Build exact-source recipe icons: python3 scripts/items/build.py --mod /path/to/oc2r."""
import argparse,hashlib,io,json,pathlib,urllib.request,zipfile
from PIL import Image,ImageChops,ImageDraw
from models import Assets,render_model,robot_model

parser=argparse.ArgumentParser()
parser.add_argument('--mod',type=pathlib.Path,required=True)
parser.add_argument('--client',type=pathlib.Path,default=pathlib.Path.home()/'.gradle/caches/neoformruntime/artifacts/minecraft_1.21.1_client.jar')
args=parser.parse_args()
root=pathlib.Path(__file__).resolve().parents[2]
out=root/'web/public/items';out.mkdir(parents=True,exist_ok=True)
assets=Assets(args.mod/'src/main/resources',zipfile.ZipFile(args.client))
tags={
 'c:chests/wooden':'minecraft:chest','c:dusts/redstone':'minecraft:redstone',
 'c:ender_pearls':'minecraft:ender_pearl','c:gems/diamond':'minecraft:diamond',
 'c:gems/emerald':'minecraft:emerald','c:gems/quartz':'minecraft:quartz',
 'c:glass_blocks':'minecraft:glass','c:ingots/copper':'minecraft:copper_ingot',
 'c:ingots/gold':'minecraft:gold_ingot','c:ingots/iron':'minecraft:iron_ingot',
 'c:ingots/netherite':'minecraft:netherite_ingot','c:sands':'minecraft:sand',
 'c:silicon':'oc2r:silicon','c:storage_blocks/redstone':'minecraft:redstone_block',
 'c:strings':'minecraft:string','minecraft:buttons':'minecraft:oak_button',
 'oc2r:wrenches':'oc2r:wrench'}
tag_names={
 'c:chests/wooden':{'en':'Wooden chests','ru':'Деревянные сундуки'},
 'c:dusts/redstone':{'en':'Redstone dust','ru':'Красная пыль'},
 'c:ender_pearls':{'en':'Ender pearls','ru':'Эндер-жемчуг'},
 'c:gems/diamond':{'en':'Diamonds','ru':'Алмазы'},
 'c:gems/emerald':{'en':'Emeralds','ru':'Изумруды'},
 'c:gems/quartz':{'en':'Quartz gems','ru':'Кварц'},
 'c:glass_blocks':{'en':'Glass blocks','ru':'Стеклянные блоки'},
 'c:ingots/copper':{'en':'Copper ingots','ru':'Медные слитки'},
 'c:ingots/gold':{'en':'Gold ingots','ru':'Золотые слитки'},
 'c:ingots/iron':{'en':'Iron ingots','ru':'Железные слитки'},
 'c:ingots/netherite':{'en':'Netherite ingots','ru':'Незеритовые слитки'},
 'c:sands':{'en':'Sand','ru':'Песок'},
 'c:silicon':{'en':'Silicon','ru':'Кремний'},
 'c:storage_blocks/redstone':{'en':'Redstone blocks','ru':'Блоки красного камня'},
 'c:strings':{'en':'String','ru':'Нити'},
 'minecraft:buttons':{'en':'Buttons','ru':'Кнопки'},
 'oc2r:wrenches':{'en':'Wrenches','ru':'Гаечные ключи'}}
ids=set(tags.values());required_tags=set()
def walk(obj):
    if isinstance(obj,dict):
        for k,v in obj.items():
            if k in ('item','id') and isinstance(v,str):ids.add(v)
            if k=='tag':required_tags.add(v)
            walk(v)
    elif isinstance(obj,list):
        for v in obj:walk(v)
recipes=list((args.mod/'src/main/resources/data/oc2r/recipe').glob('*.json'))
for recipe in recipes:walk(json.loads(recipe.read_text()))
for model in (args.mod/'src/main/resources/assets/oc2r/models/item').glob('*.json'):
    if model.stem!='monitor':ids.add('oc2r:'+model.stem)
assert required_tags<=tags.keys(),required_tags-tags.keys()
langs={}
for lang,filename in [('en','en_us'),('ru','ru_ru')]:
    langs[lang]={}
    for ns in ('minecraft','oc2r'):
        try:langs[lang][ns]=json.loads(assets.read(ns,'lang/'+filename+'.json'))
        except KeyError:langs[lang][ns]={}
# English is in the game jar; Russian is an official language-pack asset mirrored by mcasset.
urls=[]
for lang,filename in [('ru','ru_ru')]:
    if not langs[lang]['minecraft']:
        url='https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.1/assets/minecraft/lang/'+filename+'.json'
        try:
            data=urllib.request.urlopen(url,timeout=40).read()
            langs[lang]['minecraft']=json.loads(data);urls.append(url)
        except Exception as error:raise RuntimeError('Cannot obtain Minecraft translations') from error
colors={'hard_drive_small':0x898989,'hard_drive_medium':0x1c9c31,'hard_drive_large':0x11c5bd,
 'hard_drive_extra_large':0xfffc49,'hard_drive_onyxos':0xdd3d3d,'floppy':0x745c42,'floppy_modern':0x745c42}
manifest={'minecraft_version':'1.21.1','items':{},'tags':tags,'tag_names':tag_names}
for identifier in sorted(ids):
    ns,name=identifier.split(':');model=assets.model(identifier.replace(':',':item/'))
    if identifier=='oc2r:robot':model=robot_model()
    if identifier=='minecraft:chest':
        # Vanilla chest is a block-entity model, absent from JSON. Match its true 14px cube + latch.
        model={'textures':{'chest':'minecraft:entity/chest/normal'},'elements':[]}
        for lo,hi,uv in [([1,0,1],[15,10,15],{'north':[14,33,28,43],'south':[42,33,56,43],'east':[28,33,42,43],'west':[0,33,14,43],'up':[14,19,28,33]}),
         ([1,10,1],[15,14,15],{'north':[14,14,28,18],'south':[42,14,56,18],'east':[28,14,42,18],'west':[0,14,14,18],'up':[14,0,28,14]}),
         ([7,7,0],[9,11,1],{'north':[1,1,3,5],'south':[4,1,6,5],'east':[3,1,4,5],'west':[0,1,1,5],'up':[1,0,3,1]})]:
            model['elements'].append({'from':lo,'to':hi,'faces':{s:{'texture':'#chest','uv':[v/4 for v in coords]} for s,coords in uv.items()}})
    if 'layer0' in model.get('textures',{}):
        image=Image.new('RGBA',(128,128))
        for layer,texture in sorted(model['textures'].items()):
            if not layer.startswith('layer'):continue
            while texture.startswith('#'):texture=model['textures'][texture[1:]]
            piece=assets.texture(texture).resize((128,128),Image.Resampling.NEAREST)
            if layer=='layer1' and name in colors:
                color=colors[name];tint=Image.new('RGBA',piece.size,((color>>16)&255,(color>>8)&255,color&255,255))
                piece=ImageChops.multiply(piece,tint)
            image.alpha_composite(piece)
    else:image=render_model(model,assets)
    assert image.getbbox(),identifier+' rendered blank'
    dest=out/ns/(name+'.png');dest.parent.mkdir(parents=True,exist_ok=True);image.save(dest,optimize=True)
    names={}
    for lang in ('en','ru'):
        language=langs[lang][ns]
        names[lang]=language.get('item.'+ns+'.'+name,language.get('block.'+ns+'.'+name,name.replace('_',' ').title()))
        for base in ('memory','hard_drive'):
            if name in [base+'_'+tier for tier in ('small','medium','large','extra_large')]:
                tier=name[len(base)+1:];sizes={'en':{'small':'Small','medium':'Medium','large':'Large','extra_large':'Extra Large'},'ru':{'small':'малый','medium':'средний','large':'большой','extra_large':'сверхбольшой'}}
                names[lang]=language['item.oc2r.'+base]+' ('+sizes[lang][tier]+')'
        extra={'network_switch':{'en':'Network Switch','ru':'Сетевой коммутатор'},'pci_card_cage':{'en':'PCI Card Cage','ru':'Корзина PCI-карт'},'vxlan_hub':{'en':'VXLAN Hub','ru':'VXLAN-концентратор'}}
        if ns=='oc2r' and name in extra:names[lang]=extra[name][lang]
    manifest['items'][identifier]={'path':ns+'/'+name+'.png','name':names}
(out/'manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
(out/'sources.json').write_text(json.dumps({'minecraft_version':'1.21.1','minecraft':{'copyright':'Mojang Studios / Microsoft; original game assets retain their rights','catalog':'https://mcasset.cloud/1.21.1/','asset_origin':'https://assets.mcasset.cloud/1.21.1/','client_jar_sha256':hashlib.sha256(args.client.read_bytes()).hexdigest(),'downloaded_urls':urls+assets.urls},'oc2r':{'source':'https://github.com/DivByDiamond/OC2R','path':'src/main/resources/assets/oc2r','license':'See OC2R project license; original textures preserved'},'renderer':'Orthographic JSON geometry, GUI transform, nearest texture sampling; robot uses RobotModel.java cube geometry, chest uses vanilla chest geometry and texture UVs. Software shading approximates in-game GUI lighting.'},indent=2)+'\n')
sheet=Image.new('RGB',(800,((len(ids)+7)//8)*116),'#c0c0c0');draw=ImageDraw.Draw(sheet)
for i,identifier in enumerate(sorted(ids)):
    x=i%8*100;y=i//8*116;ns,name=identifier.split(':')
    im=Image.open(out/ns/(name+'.png')).resize((80,80),Image.Resampling.NEAREST)
    sheet.paste(im,(x+10,y),im);draw.text((x+1,y+82),name[:16],fill='black')
sheet.save(out/'contact-sheet.png')
print(f'Built {len(ids)} icons; {len(recipes)} recipes, {len(required_tags)} tags covered.')
