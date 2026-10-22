use crate::admin::common::{SEED_QUOTA, env, login};
use crate::support::save_page;
use oc2r_wiki::domain::page::Lang;

const XSS: &str = "<script>alert(1)</script>";
const ESC: &str = "&#60;script&#62;alert(1)&#60;/script&#62;";

fn url_enc(s: &str) -> String {
    s.bytes().map(|b| format!("%{b:02X}")).collect()
}

#[tokio::test]
async fn page_summary_and_author_are_escaped_in_the_editor_history() {
    let e = env();
    save_page(&e.st, "a", Lang::En, "v2", XSS);
    let html = e.get_text("/admin/edit/en/a").await;
    assert!(!html.contains(XSS), "{html}");
    assert!(html.contains(ESC));
}

#[tokio::test]
async fn editor_textarea_cannot_be_broken_out_of() {
    let e = env();
    save_page(&e.st, "a", Lang::En, "</textarea><b id=pwn>x</b>", "s");
    let html = e.get_text("/admin/edit/en/a").await;
    assert!(!html.contains("</textarea><b"), "{html}");
    assert!(html.contains("&#60;/textarea&#62;"));
    assert_eq!(html.matches("</textarea>").count(), 1);
}

#[tokio::test]
async fn key_names_are_escaped_in_list_and_creation_response() {
    let e = env();
    let r = e
        .post("/admin/keys", &format!("name={}&read=1", url_enc(XSS)))
        .await;
    let created = crate::support::text(r).await;
    assert!(!created.contains(XSS) && created.contains(ESC));
    let list = e.get_text("/admin/keys").await;
    assert!(!list.contains(XSS) && list.contains(ESC));
}

#[tokio::test]
async fn proposal_pages_escape_summary_body_and_author() {
    let e = env();
    let (_, writer) = login(&e.st, 60, XSS, false, false);
    let p = e.page("a");
    let body = "</textarea><b id=pwn>x</b><script>alert(1)</script>";
    let pid =
        e.st.proposals
            .create(p.id, p.current.id, body, XSS, writer, SEED_QUOTA)
            .unwrap()
            .unwrap();
    // make it conflicting so the merge textarea and the pre blocks are shown
    save_page(&e.st, "a", Lang::En, "v2", "other");
    for uri in [
        "/admin/proposals".to_owned(),
        format!("/admin/proposals/{pid}"),
    ] {
        let html = e.get_text(&uri).await;
        assert!(!html.contains(XSS), "{uri}");
        assert!(!html.contains("</textarea><b"), "{uri}");
        assert!(!html.contains("<b id=pwn>"), "{uri}");
    }
    let html = e.get_text(&format!("/admin/proposals/{pid}")).await;
    assert!(html.contains("&#60;/textarea&#62;"));
}

#[tokio::test]
async fn user_logins_are_escaped_in_the_users_table() {
    let e = env();
    login(&e.st, 61, XSS, false, false);
    let html = e.get_text("/admin/users").await;
    assert!(!html.contains(XSS) && html.contains(ESC));
}
