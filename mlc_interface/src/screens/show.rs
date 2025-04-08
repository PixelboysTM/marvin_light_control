use dioxus::prelude::*;
use crate::utils::Panel;

#[component]
pub fn Show() -> Element {
    rsx! {
        div{
            class: "configure",
            Panel {
                column: "1 / 4",
                row: "1 / 13",
                title: "Effect Library"
            }
            Panel {
                column: "4 / 13",
                row: "1 / 10",
                title: "Effect Shortcuts"
            }
            Panel {
                column: "4 / 13",
                row: "10 / 13",
                title: "Effect Stack"
            }
        }
    }
}