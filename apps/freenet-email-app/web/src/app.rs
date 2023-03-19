#![allow(non_snake_case)]
use std::{borrow::Cow, cell::RefCell, rc::Rc};

use chrono::Utc;
use dioxus::prelude::*;
use locutus_stdlib::prelude::ContractContainer;
use rsa::{pkcs1::DecodeRsaPrivateKey, RsaPrivateKey, RsaPublicKey};

use crate::inbox::{DecryptedMessage, InboxModel, MessageModel};

mod login;

#[derive(Debug, Clone)]
struct Inbox {
    inbox_ids: Vec<Identity>,
    inbox_data: Vec<Rc<RefCell<InboxModel>>>,
    messages: Rc<RefCell<Vec<Message>>>,
    active_id: usize,
}

impl Inbox {
    fn new(
        cx: Scope,
        contracts: Vec<Identity>,
        private_key: &rsa::RsaPrivateKey,
    ) -> Result<Self, String> {
        let mut models = Vec::with_capacity(contracts.len());
        #[cfg(feature = "use-node")]
        {
            for contract in &contracts {
                let model = InboxModel::load(cx, contract, private_key)?;
                models.push(Rc::new(RefCell::new(model)));
            }
        }
        Ok(Self {
            inbox_data: models,
            inbox_ids: contracts,
            messages: Rc::new(RefCell::new(vec![])),
            active_id: 0,
        })
    }

    fn send_message(
        &self,
        cx: Scope,
        to: &str,
        title: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("adding to {}", self.active_id);
        #[cfg(feature = "use-node")]
        {
            let content = DecryptedMessage {
                title: title.to_owned(),
                content: content.to_owned(),
                from: "".to_owned(),
                to: vec![to.to_owned()],
                cc: vec![],
                time: Utc::now(),
            };
            #[cfg(target_arch = "wasm32")]
            {
                web_sys::console::log_1(
                    &serde_wasm_bindgen::to_value(&format!("adding to {}", self.active_id))
                        .unwrap(),
                );
            }

            async fn get_inbox(key: &str) -> Result<InboxModel, Box<dyn std::error::Error>> {
                todo!()
            }

            for k in content.to.iter() {
                let inbox = futures::executor::block_on(get_inbox(k))?;
                InboxModel::send_message(inbox, cx, content.clone())?;
            }
        }
        let _ = cx;
        Ok(())
    }

    fn remove_messages<T>(
        &self,
        cx: Scope<T>,
        ids: &[u64],
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("removing {}", self.active_id);
        #[cfg(feature = "use-node")]
        {
            let inbox = self.inbox_data[self.active_id].clone();
            InboxModel::remove_messages(inbox, cx, ids)?;
        }
        Ok(())
    }

    // Remove the messages from the inbox contract, and move them to local storage
    fn mark_as_read<T>(&self, cx: Scope<T>, ids: &[u64]) -> Result<(), Box<dyn std::error::Error>> {
        let messages = &mut *self.messages.borrow_mut();
        let mut removed_messages = Vec::with_capacity(ids.len());
        for e in messages {
            if ids.contains(&e.id) {
                e.read = true;
                let m = e.clone();
                removed_messages.push(m);
            }
        }
        // todo: persist in a sidekick `removed_messages`
        self.remove_messages(cx, ids)?;
        Ok(())
    }

    #[cfg(feature = "ui-testing")]
    fn load_messages(&self, _cx: Scope, id: &Identity, _private_key: &rsa::RsaPrivateKey) {
        let emails = {
            if id.id == 0 {
                vec![
                    Message {
                        id: 0,
                        from: "Ian's Other Account".into(),
                        title: "Email from Ian's Other Account".into(),
                        content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit..."
                            .repeat(10)
                            .into(),
                        read: false,
                    },
                    Message {
                        id: 1,
                        from: "Mary".to_string().into(),
                        title: "Email from Mary".to_string().into(),
                        content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit..."
                            .repeat(10)
                            .into(),
                        read: false,
                    },
                ]
            } else {
                vec![
                    Message {
                        id: 0,
                        from: "Ian Clarke".into(),
                        title: "Email from Ian".into(),
                        content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit..."
                            .repeat(10)
                            .into(),
                        read: false,
                    },
                    Message {
                        id: 1,
                        from: "Jane".to_string().into(),
                        title: "Email from Jane".to_string().into(),
                        content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit..."
                            .repeat(10)
                            .into(),
                        read: false,
                    },
                ]
            }
        };
        self.messages.replace(emails);
    }

    #[cfg(all(feature = "use-node", not(feature = "ui-testing")))]
    fn load_messages(&self, cx: Scope, id: &Identity, private_key: &rsa::RsaPrivateKey) {
        CONNECTION.with(|conn| {
            let private_key = private_key.clone();
            let key = self
                .contracts
                .iter()
                .find(|c| c.id == id.id)
                .unwrap()
                .pub_key
                .clone();
            let contract_key = todo!("get the id, from the code + params");
            let client = (**conn).clone();
            let f = use_future(cx, (), |_| async move {
                let client = &mut *client.borrow_mut();
                InboxModel::get_inbox(client, &private_key, contract_key).await
            });
            let inbox = loop {
                match f.value() {
                    Some(v) => break v.as_ref().unwrap(),
                    None => std::thread::sleep(std::time::Duration::from_millis(100)),
                }
            };
            let messages = &mut *self.messages.borrow_mut();
            messages.clear();
            messages.extend(inbox.messages.iter().map(|m| m.clone().into()));
        })
    }
}

struct User {
    logged: bool,
    identified: bool,
    active_id: Option<usize>,
    identities: Vec<Identity>,
    private_key: Option<RsaPrivateKey>,
}

impl User {
    #[cfg(feature = "ui-testing")]
    fn new() -> Self {
        const RSA_4096_PRIV_PEM: &str = include_str!("../examples/rsa4096-id-1-priv.pem");
        let priv_key = RsaPrivateKey::from_pkcs1_pem(RSA_4096_PRIV_PEM).unwrap();
        // TODO: here we should be checking if an existing identity sidekick exists
        let identified = true;
        User {
            logged: false,
            identified,
            active_id: None,
            identities: vec![
                Identity {
                    alias: "ian.clarke@freenet.org".to_owned(),
                    id: 0,
                    pub_key: priv_key.to_public_key(),
                },
                Identity {
                    alias: "other.stuff@freenet.org".to_owned(),
                    id: 1,
                    pub_key: priv_key.to_public_key(),
                },
            ],
            private_key: Some(priv_key),
        }
    }

    fn logged_id(&self) -> Option<&Identity> {
        self.active_id.and_then(|id| self.identities.get(id))
    }

    fn set_logged_id(&mut self, id: usize) {
        assert!(id < self.identities.len());
        self.active_id = Some(id);
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Identity {
    pub id: usize,
    pub pub_key: RsaPublicKey,
    alias: String,
}

#[derive(Debug, Clone, Eq, Props)]
struct Message {
    id: u64,
    from: Cow<'static, str>,
    title: Cow<'static, str>,
    content: Cow<'static, str>,
    read: bool,
}

impl From<MessageModel> for Message {
    fn from(value: MessageModel) -> Self {
        Message {
            id: value.id,
            from: value.content.from.into(),
            title: value.content.title.into(),
            content: value.content.content.into(),
            read: false,
        }
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub(crate) fn App(cx: Scope) -> Element {
    // #[cfg(target_arch = "wasm32")]
    // {
    //     web_sys::console::log_1(&serde_wasm_bindgen::to_value("Starting app...").unwrap());
    // }
    // TODO: in the future this will be dinamically loaded from the identity component
    let contracts = vec![
        Identity {
            id: 0,
            alias: "ian.clarke@freenet.org".to_owned(),
            pub_key: {
                const RSA_PRIV_PEM: &str = include_str!("../examples/rsa4096-id-1-priv.pem");
                RsaPrivateKey::from_pkcs1_pem(RSA_PRIV_PEM)
                    .unwrap()
                    .to_public_key()
            },
        },
        Identity {
            id: 1,
            alias: "other.stuff@freenet.org".to_owned(),
            pub_key: {
                const RSA_PRIV_PEM: &str = include_str!("../examples/rsa4096-id-2-priv.pem");
                RsaPrivateKey::from_pkcs1_pem(RSA_PRIV_PEM)
                    .unwrap()
                    .to_public_key()
            },
        },
    ];
    use_shared_state_provider(cx, User::new);
    use_context_provider(cx, || {
        const RSA_PRIV_PEM: &str = include_str!("../examples/rsa4096-user-priv.pem");
        let key = RsaPrivateKey::from_pkcs1_pem(RSA_PRIV_PEM).unwrap();
        Inbox::new(cx, contracts, &key).unwrap()
    });

    let user = use_shared_state::<User>(cx).unwrap();
    if !user.read().identified {
        cx.render(rsx! {
            login::GetOrCreateIndentity {}
        })
    } else if let Some(id) = user.read().logged_id() {
        let inbox = use_context::<Inbox>(cx).unwrap();
        inbox.load_messages(cx, id, user.read().private_key.as_ref().unwrap());
        cx.render(rsx! {
           UserInbox {}
        })
    } else {
        cx.render(rsx! {
           login::IdentifiersList {}
        })
    }
}

mod menu {
    #[derive(Default)]
    pub(super) struct MenuSelection {
        email: Option<u64>,
        new_msg: bool,
    }

    impl MenuSelection {
        pub fn at_new_msg(&mut self) {
            if self.new_msg {
                self.new_msg = false;
            } else {
                self.new_msg = true;
                self.email = None;
            }
        }

        pub fn is_new_msg(&self) -> bool {
            self.new_msg
        }

        pub fn at_inbox_list(&mut self) {
            self.email = None;
            self.new_msg = false;
        }

        pub fn is_received(&self) -> bool {
            !self.new_msg && self.email.is_none()
        }

        pub fn open_email(&mut self, id: u64) {
            self.email = Some(id);
        }

        pub fn email(&self) -> Option<u64> {
            self.email
        }
    }
}

fn UserInbox(cx: Scope) -> Element {
    use_shared_state_provider(cx, menu::MenuSelection::default);
    cx.render(rsx!(
        div {
            class: "columns",
            nav {
                class: "column is-one-fifth menu",
                UserMenuComponent {}
            }
            div {
                class: "column",
                InboxComponent {}
            }
        }
    ))
}

fn UserMenuComponent(cx: Scope) -> Element {
    let user = use_shared_state::<User>(cx).unwrap();
    let menu_selection = use_shared_state::<menu::MenuSelection>(cx).unwrap();

    let received_class = (menu_selection.read().is_received()
        || !menu_selection.read().is_new_msg())
    .then(|| "is-active")
    .unwrap_or("");
    let write_msg_class = menu_selection
        .read()
        .is_new_msg()
        .then(|| "is-active")
        .unwrap_or("");

    cx.render(rsx!(
        div {
            class: "pl-3 pr-3 mt-3",
            ul {
                class: "menu-list",
                li {
                    a {
                        class: received_class,
                        onclick: move |_| { menu_selection.write().at_inbox_list(); },
                        "Received"
                    }
                }
                li {
                    a {
                        class: write_msg_class,
                        onclick: move |_| {
                            let mut selection = menu_selection.write();
                            selection.at_new_msg();
                        },
                        "Write message"
                    }
                }
                li {
                    a {
                        onclick: move |_| {
                            let mut logged_state = user.write();
                            logged_state.logged = false;
                            logged_state.active_id = None;
                        },
                        "Log out"
                    }
                }
            }
        }
    ))
}

fn InboxComponent(cx: Scope) -> Element {
    let inbox = use_context::<Inbox>(cx).unwrap();
    let menu_selection = use_shared_state::<menu::MenuSelection>(cx).unwrap();

    #[inline_props]
    fn EmailLink<'a>(
        cx: Scope<'a>,
        sender: Cow<'a, str>,
        title: Cow<'a, str>,
        read: bool,
        id: u64,
    ) -> Element {
        let open_mail = use_shared_state::<menu::MenuSelection>(cx).unwrap();
        let icon_style = read
            .then(|| "fa-regular fa-envelope")
            .unwrap_or("fa-solid fa-envelope");
        cx.render(rsx!(a {
            class: "panel-block",
            id: "email-inbox-accessor-{id}",
            onclick: move |_| { open_mail.write().open_email(*id); },
            span {
                class: "panel-icon",
                i { class: icon_style }
            }
            span { class: "ml-2", "{sender}" }
            span { class: "ml-5", "{title}" }
        }))
    }

    let emails = inbox.messages.borrow();
    let is_email = menu_selection.read().email();
    if let Some(email_id) = is_email {
        let id_p = (*emails).binary_search_by_key(&email_id, |e| e.id).unwrap();
        let email = &emails[id_p];
        cx.render(rsx! {
            OpenMessage {
                id: email.id,
                from: email.from.clone(),
                title: email.title.clone(),
                content: email.content.clone(),
                read: email.read,
            }
        })
    } else if menu_selection.read().is_new_msg() {
        cx.render(rsx! {
            NewMessageWindow {}
        })
    } else {
        let links = emails.iter().map(|email| {
            rsx!(EmailLink {
                sender: email.from.clone(),
                title: email.title.clone()
                read: email.read,
                id: email.id,
            })
        });
        cx.render(rsx! {
            div {
                class: "panel is-link mt-3",
                p { class: "panel-heading", "Inbox" }
                p {
                    class: "panel-tabs",
                    a {
                        class: "is-active icon-text",
                        span { class: "icon", i { class: "fas fa-inbox" } }
                        span { "Primary" }
                    }
                    a {
                        class: "icon-text",
                        span { class: "icon",i { class: "fas fa-user-group" } },
                        span { "Social" }
                    }
                    a {
                        class: "icon-text",
                        span { class: "icon", i { class: "fas fa-circle-exclamation" } },
                        span { "Updates" }
                    }
                }
                div {
                    class: "panel-block",
                    p {
                        class: "control has-icons-left",
                        input { class: "input is-link", r#type: "text", placeholder: "Search" }
                        span { class: "icon is-left", i { class: "fas fa-search", aria_hidden: true } }
                    }
                }
                links
            }
        })
    }
}

fn OpenMessage(cx: Scope<Message>) -> Element {
    let menu_selection = use_shared_state::<menu::MenuSelection>(cx).unwrap();
    let inbox = use_context::<Inbox>(cx).unwrap();
    let email = cx.props;
    match inbox.mark_as_read(cx, &[email.id]) {
        Ok(()) => {}
        Err(e) => {
            let err = format!("{e}");
            #[cfg(all(feature = "use-node", target_arch = "wasm32"))]
            {
                web_sys::console::error_1(&serde_wasm_bindgen::to_value(&err).unwrap());
            }
            tracing::error!("error while updating message state: {err}");
        }
    }
    cx.render(rsx! {
        div {
            class: "columns title mt-3",
            div {
                class: "column",
                a {
                    class: "icon is-small",
                    onclick: move |_| {
                        menu_selection.write().at_inbox_list();
                    },
                    i { class: "fa-sharp fa-solid fa-arrow-left", aria_label: "Back to Inbox", style: "color:#4a4a4a" }, 
                }
            }
            div { class: "column is-four-fifths", h2 { "{email.title}" } }
            div {
                class: "column", 
                a {
                    class: "icon is-small", 
                    onclick: move |_| {
                        match inbox.remove_messages(cx, &[email.id]) {
                            Ok(()) => {}
                            Err(e) => {
                                let err = format!("{e}");
                                #[cfg(all(feature = "use-node", target_arch = "wasm32"))]
                                {
                                    web_sys::console::error_1(&serde_wasm_bindgen::to_value(&err).unwrap());
                                }
                                tracing::error!("error while deleting message: {err}");
                            }
                        }
                        menu_selection.write().at_inbox_list();
                    },
                    i { class: "fa-sharp fa-solid fa-trash", aria_label: "Delete", style: "color:#4a4a4a" } 
                }
            }
        }
        div {
            id: "email-content-{email.id}",
            p {
                "{email.content}"
            }
        }
    })
}

fn NewMessageWindow(cx: Scope) -> Element {
    let menu_selection = use_shared_state::<menu::MenuSelection>(cx).unwrap();
    let inbox = use_context::<Inbox>(cx).unwrap();
    let user = use_shared_state::<User>(cx).unwrap();
    let user = user.read();
    let user_alias = user.logged_id().unwrap().alias.as_str();
    let to = use_state(cx, String::new);
    let title = use_state(cx, String::new);
    let content = use_state(cx, String::new);
    cx.render(rsx! {
        div {
            class: "column mt-3",
            div {
                class: "box has-background-light",
                h3 { class: "title is-3", "New message" }
                table {
                    class: "table is-narrow has-background-light",
                    tbody {
                        tr {
                            th { "From" }
                            td { style: "width: 100%", "{user_alias}" }
                        }
                        tr {
                            th { "To"}
                            td { style: "width: 100%", contenteditable: true, "{to}" }
                        }
                        tr {
                            th { "Title"}
                            td { style: "width: 100%", contenteditable: true, "{title}"  }
                        }
                    }
                }
            }
            div {
                class: "box",
                div {
                    contenteditable: true,
                    oninput: move |ev| { content.set(ev.value.clone()); },
                    br {}
                }
            }
            div {
                button {
                    class: "button is-info is-outlined",
                    onclick: move |_| {
                        match inbox.send_message(cx, to.get(), title.get(), content.get()) {
                            Ok(()) => {}
                            Err(e) => {
                                let err = format!("{e}");
                                #[cfg(all(feature = "use-node", target_arch = "wasm32"))]
                                {
                                    web_sys::console::error_1(&serde_wasm_bindgen::to_value(&err).unwrap());
                                }
                                tracing::error!("error while sending message: {err}");
                            }
                        }
                        menu_selection.write().at_new_msg();
                    },
                    "Send"
                }
            }
        }
    })
}
