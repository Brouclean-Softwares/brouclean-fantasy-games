use askama::Template;
use askama_web::WebTemplate;

#[derive(Template, WebTemplate)]
#[template(path = "shared/modal_button.html")]
pub struct ModalButton {
    button_level: &'static str,
    button_name: &'static str,
    modal_id: &'static str,
    modal_title: &'static str,
    modal_content: String,
    action_button_name: &'static str,
    form_action: &'static str,
}

impl ModalButton {
    pub fn from(
        button_level: &'static str,
        button_name: &'static str,
        modal_id: &'static str,
        modal_title: &'static str,
        modal_content: String,
        action_button_name: &'static str,
        form_action: &'static str,
    ) -> ModalButton {
        ModalButton {
            button_level,
            button_name,
            modal_id,
            modal_title,
            modal_content,
            action_button_name,
            form_action,
        }
    }

    pub fn delete_button_from(delete_url: &'static str, element_id: i64) -> ModalButton {
        ModalButton::from(
            "danger",
            "Supprimer",
            "delete",
            "Suppression en cours",
            DeleteModalButtonContent { element_id }.render().unwrap(),
            "Supprimer",
            delete_url,
        )
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "shared/delete_modal_button_content.html")]
struct DeleteModalButtonContent {
    element_id: i64,
}
