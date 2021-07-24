use sauron::prelude::*;

pub struct MenuItem {
    is_active: bool,
    href: &'static str,
    title: &'static str,
}

impl MenuItem {
    pub fn render<T>(&self) -> Node<T> {
        let mut attr = vec![href(self.href), class("menu-item")];
        if self.is_active {
            attr.push(class("active"));
        }
        a(attr, vec![text(self.title)])
    }
}

const TITLE: &'static str = "API3 DAO Tracker";
const SLOGAN: &'static str = "on-chain analytics: DAO members, rewards, token supply";

pub fn render<T>(active_menu: &'static str) -> Node<T> {
    let menu: Vec<MenuItem> = vec![
        MenuItem {
            href: "./",
            title: "Statistics",
            is_active: false,
        },
        MenuItem {
            href: "./wallets",
            title: "Wallets",
            is_active: active_menu.starts_with("/wallets"),
        },
        MenuItem {
            href: "./votings",
            title: "Votings",
            is_active: active_menu.starts_with("/votings"),
        },
    ];

    node! {
      <header>
        <div class="inner">
          <div class="nav-brand">
            <span class="nav-brand__label">
              {text(TITLE)}
            </span>
            <span class="nav-brand__slogan">
              {text(SLOGAN)}
            </span>
          </div>
          <div class="mid"></div>
          {
            div(
              vec![class("desktop-menu")],
              menu.iter().map(|x| x.render()).collect::<Vec<Node<T>>>(),
            )
          }
        </div>
      </header>
    }
}
