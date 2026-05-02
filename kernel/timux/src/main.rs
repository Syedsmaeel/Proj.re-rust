use timux::bashpp::lexer::Lexer;
use timux::bashpp::parser::{Parser, WidgetKind, BorderStyle};
use timux::bashpp::runtime::Runtime;
use timux::bashpp::compat::{BashCompat, is_bash_script};
use timux::bashpp::tui::{TuiShell, Tab, Pane, Widget, Theme, Cell, color};

fn sep(title: &str) {
    println!("\n\x1b[96m\x1b[1m> {title}\x1b[0m");
}

fn run(src: &str) -> alloc::vec::Vec<alloc::string::String> {
    let mut lex = Lexer::new(src);
    let toks = lex.tokenize();
    let mut p = Parser::new(toks);
    let ast = p.parse();
    let mut rt = Runtime::new();
    rt.exec_block(&ast);
    rt.output.drain()
}

extern crate alloc;

fn main() {
    println!("\n\x1b[1m\x1b[96m=== Bash++ live demo ===\x1b[0m");

    sep("1. Lexer");
    let src = r#"let x := 42; echo "hello timux" | grep foo && ls"#;
    let mut lex = Lexer::new(src);
    let toks = lex.tokenize();
    println!("  source: \x1b[2m{src}\x1b[0m");
    println!("  {} tokens", toks.len());
    for t in toks.iter().take(8) {
        print!("  \x1b[92m[{:?}]\x1b[0m", t.kind);
    }
    println!(" ...");

    sep("2. Bash++ native runtime");
    let prog = concat!(
        "let name := \"Timux\"\n",
        "echo $name\n",
        "render \"hello from bashpp\" color=cyan\n",
        "icon \"🦀\"\n",
        "theme dracula\n",
    );
    let out = run(prog);
    for l in &out { println!("  \x1b[93m>\x1b[0m {l}"); }

    sep("3. Bash compat layer");
    let bash = "#!/bin/bash\nfunction hi() { echo \"hello from bash\"; }\nhi\nexport FOO=bar\necho done";
    println!("  is_bash: {}", is_bash_script(bash));
    let res = BashCompat::run(bash);
    println!("  exit_code: {}", res.exit_code);
    for l in &res.output { println!("  \x1b[92m>\x1b[0m {l}"); }

    sep("4. TUI shell render");
    let mut shell = TuiShell::new(60, 24);
    shell.set_theme("dracula");

    let mut tab1 = Tab::new("shell");
    tab1.icon = Some(alloc::string::String::from("🐚"));
    tab1.active = true;
    let mut pane = Pane::new(1, 58, 8);
    pane.title = Some(alloc::string::String::from("Bash++ REPL"));
    pane.border = BorderStyle::Rounded;
    pane.cells.push(Cell::colored("bashpp> echo hello", color::CYAN));
    pane.cells.push(Cell::plain("hello"));
    pane.cells.push(Cell::colored("bashpp> theme dracula", color::CYAN));
    pane.cells.push(Cell::plain("[theme:dracula]"));
    tab1.add_pane(pane);
    shell.add_tab(tab1);

    let mut tab2 = Tab::new("files");
    tab2.icon = Some(alloc::string::String::from("📁"));
    tab2.add_pane(Pane::new(2, 58, 8));
    shell.add_tab(tab2);

    let mut tab3 = Tab::new("proc");
    tab3.icon = Some(alloc::string::String::from("⚙"));
    tab3.add_pane(Pane::new(3, 58, 8));
    shell.add_tab(tab3);

    for line in shell.render() { println!("{line}"); }

    sep("5. Widgets");
    let theme = Theme::Dracula;
    let widgets = alloc::vec![
        Widget::new(WidgetKind::Clock, 40, 4),
        Widget::new(WidgetKind::FileTree { path: alloc::string::String::from("/timux") }, 40, 6),
        Widget::new(WidgetKind::ProcessViewer, 40, 6),
    ];
    for w in &widgets {
        for l in w.render(&theme) { println!("{l}"); }
        println!();
    }

    println!("\n\x1b[1m\x1b[92m  Bash++ compiled + running\x1b[0m");
    println!("\x1b[2m  lexer parser runtime tui compat — all live\x1b[0m\n");
}
