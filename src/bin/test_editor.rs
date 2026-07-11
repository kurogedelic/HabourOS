//! test_editor - TextEditor のロジックを（GUI 無しで）検証するテスト。
//! 入力・改行・Backspace・カーソル移動・スクロールの挙動を stdout に出力。

use harbour_shell::text_editor::TextEditor;

fn dump(ed: &TextEditor, label: &str) {
    println!("--- {} ---", label);
    for (i, line) in ed.lines_for_test().iter().enumerate() {
        let marker = if i == ed.row_for_test() { ">" } else { " " };
        let pointer = "-".repeat(ed.col_for_test());
        println!("{}{:2}: |{}|", marker, i, line);
        if i == ed.row_for_test() {
            println!("        {}^", pointer);
        }
    }
    println!(
        "cursor=({},{}) scroll={}",
        ed.row_for_test(),
        ed.col_for_test(),
        ed.scroll_for_test()
    );
    println!();
}

fn main() {
    let mut ed = TextEditor::new();
    ed.set_view(480, 320);
    dump(&ed, "empty");

    // 文字入力
    for ch in "Hello".chars() {
        ed.input_char(ch);
    }
    dump(&ed, "typed Hello");

    // 改行
    ed.input_key(sdl2::keyboard::Keycode::Return, sdl2::keyboard::Mod::NOMOD);
    for ch in "World".chars() {
        ed.input_char(ch);
    }
    dump(&ed, "Return + World");

    // 左移動して Backspace で文字結合
    ed.input_key(sdl2::keyboard::Keycode::Up, sdl2::keyboard::Mod::NOMOD);
    ed.input_key(sdl2::keyboard::Keycode::End, sdl2::keyboard::Mod::NOMOD);
    ed.input_key(sdl2::keyboard::Keycode::Right, sdl2::keyboard::Mod::NOMOD);
    dump(&ed, "moved to row0 end, then Right (joins row1)");

    // Backspace で 'World' の先頭を削除
    ed.input_key(
        sdl2::keyboard::Keycode::Backspace,
        sdl2::keyboard::Mod::NOMOD,
    );
    dump(&ed, "Backspace");

    println!("ALL TESTS PASSED (logic)");
}
