use dioxus::prelude::*;
use crate::utils::Panel;

#[component]
pub fn Program() -> Element {
    rsx! {
        div{
            class: "configure",
            Panel {
                column: "1 / 4",
                row: "1 / 13",
                title: "Effect Explorer"
            }
            Panel {
                column: "10 / 13",
                row: "1 / 9",
                title: "Effect Inspector"
            }
            Panel {
                column: "4 / 10",
                row: "1 / 9",
                title: "3D Viewer"
            }
            Panel {
                column: "4 / 13",
                row: "9 / 13",
                title: "Timeline"
            }
        }
    }
}