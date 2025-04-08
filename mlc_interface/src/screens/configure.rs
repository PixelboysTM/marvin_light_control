use dioxus::prelude::*;
use crate::utils::Panel;

#[component]
pub fn Configure() -> Element {
    rsx! {
        div{
            class: "configure",
            Panel {
                column: "1 / 4",
                row: "1 / 9",
                title: "Fixture Catalog"
            }
            Panel {
                column: "1 / 13",
                row: "9 / 13",
                title: "Fader Panel"
            }
            Panel {
                column: "10 / 13",
                row: "1 / 9",
                title: "Settings"
            }
            Panel {
                column: "4 / 10",
                row: "1 / 9",
                title: "Universe Patcher"
            }
        }
    }
}