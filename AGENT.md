# AGENT.md — Phobos IDE (God Mode continuity)

> این فایل برای شروع چت جدید است. بدون تکرار کارهای انجام‌شده ادامه بده.

## پروژه
- ریپو: `hesam-oxe/Phobos` (public) — IDE راست با egui
- مسیر لوکال: `/config/Phobos` (برنچ `main`)
- گیت‌هاب: اکانت `hesam-oxe` با `gh` لاگین است (`gh auth status` اوکی). دوباره توکن نخواه. برای پوش از `gh auth setup-git` استفاده شده.
- محیط: Ubuntu 26.04، رم 780GB، CPU قوی. Rust 1.99 در `$HOME/.cargo/bin/cargo` (`/config/.cargo/bin/cargo`).
- زبان کاربر: فارسی جواب بده، کوتاه و فنی.

## وضعیت (آخرین پوش: d96767e)
پوش شد به `origin/main`: 42 فایل، +2866 خط.
انجام‌شده (تکرار نکن):
- `Cargo.toml`: حذف `tree-sitter-yaml 0.5` ناسازگار، افزودن `regex,futures,toml,dirs,url`
- `editor/code_editor.rs`: فیکس `language_gr grammar` → `language_grammar`، پابلیک‌کردن `content/language/config/cursor_pos`، فیکس لایوتر هایلایت (پوشش کامل متن + گپ)، حذف `ui.send_event` نامعتبر
- `languages/`: پیاده‌سازی 16 استاب (go,java,c_family,csharp,assembly,ruby,php,sql,swift,kotlin,pascal,shell,web,markdown,docker,data_langs) + رجیستری کامل در `languages/mod.rs`
- `utils/config.rs`: کانفیگ TOML persistent + recent files
- `utils/crypto.rs`: hash/hex/xor/fnv1a
- `low_level/`: `hex_editor.rs` (ادیتور واقعی)، `disassembler.rs` (iced-x86)، `memory_viewer.rs`، `register_viewer.rs`
- `services/lsp/client.rs`: بازنویسی کامل JSON-RPC با Content-Length، reader_loop، diagnostics channel، did_open/did_change/completion/hover/shutdown واقعی
- `services/terminal/shell.rs`: فیکس API portable-pty 0.8 (master/slave/reader/writer جدا، spawn_blocking)
- `services/git.rs`, `services/watcher.rs`, `services/plugins.rs`: جدید
- `ui/`: `ai_panel.rs`, `build_panel.rs`, `git_panel.rs`, `debug_panel.rs`, `pty_terminal.rs` جدید؛ `ui/mod.rs` وایر شده
- `build/compiler.rs`: بدون دد‌لاک با `output()` + cargo workspace awareness + `find_cargo_manifest`
- `build/runner.rs`: بدون دد‌لاک با `output()`
- `build/debugger.rs`: نگهداری stdin/stdout جدا، GDB/MI، breakpoint live-apply
- `app.rs`: بازنویسی God Mode — مولتی‌بافر، open_folder، format، command palette اجرایی، auto-save، watcher hot-reload، پنل‌های AI/Build/Git/Debug/Hex/Disasm/Plugins، config persist

## ناتمام / بعدی
1. `cargo check` کامل هنوز سبز نشده (سرور روی دستور سنگین foreground ریست می‌شود). فقط با `background=true` اجرا کن و خروجی را از فایل بخوان؛ `foreground` با timeout بالا نزن.
2. خطاهای احتمالی باقی: `debugger.rs`، `pty_terminal` tokio runtime در egui، `watcher.rs` notify 6 API، `iced-x86` formatter API، `url` vs `lsp-types::Url`.
3. بعد از سبز شدن check: `cargo test` (تست‌های ai/*)، سپس `cargo build --release`.
4. هدف نهایی کاربر: رساندن به 100% IDE خداگونه. مراحل بعدی: DAP کامل، استریم خروجی بیلد، مینی‌مپ واقعی، LSP hover/completion وایر به ادیتور، مارکت‌پلیس.

## دستورات امن (سبک، بدون ریست)
- `git -C /config/Phobos status --short | head`
- `gh auth status`
- `sed -n '1,50p' <file>`
- برای بیلد سنگین حتما: `shell(background=true)` + خواندن فایل خروجی، نه poll تکراری.

## ممنوع
- توکن `ghp_...` را هیچ‌جا لاگ/کامیت/نمایش نده.
- کارهای لیست «انجام‌شده» را دوباره انجام نده.
- فایل جدید بی‌دلیل نساز؛ اول `edit` روی موجود.
