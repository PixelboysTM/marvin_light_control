alias s := server
alias i := interface
alias r := run

set shell := ["cmd.exe", "/c"]

default: run


server:
    cargo run --bin mlc_server
interface:
    cd ./mlc_interface && dx serve
run:
    start just server
    start just interface

docs:
    cd ./mlc_site && npm run dev
docs-build:
    cd ./mlc_site && npm run build
