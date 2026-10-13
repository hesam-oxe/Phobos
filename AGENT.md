# AGENT.md — Phobos IDE (God Mode continuity)

> این فایل برای شروع چت جدید است. بدون تکرار کارهای انجام‌شده ادامه بده.

## پروژه
- ریپو: `hesam-oxe/Phobos` (public) — IDE راست با egui
- مسیر لوکال: `/config/Phobos` (برنچ `main`)
- گیت‌هاب: اکانت `hesam-oxe` با `gh` لاگین است (`gh auth status` اوکی). دوباره توکن نخواه. برای پوش از `gh auth setup-git` استفاده شده.
- محیط: Ubuntu 26.04، رم 780GB، CPU قوی. Rust 1.99 در `$HOME/.cargo/bin/cargo` (`/config/.cargo/bin/cargo`).
- زبان کاربر: فارسی جواب بده، کوتاه و فنی.

## وضعیت (آخرین پوش: 2a739dd — چک‌سبز، پوش شد)
`cargo check -j 4` و `cargo check --tests`: سبز (فقط 2 وارنینگ deprecated).
انجام‌شده این دور:
- مارکت‌پلیس: کاتالوگ + search + install از TOML + remove + persist حالت در `state.toml`
- AI واقعی: `ai/provider.rs` (OpenAI-compatible via ureq، key فقط در حافظه) + تب Remote با endpoint/model ذخیره‌شونده
- DAP کامل: `build/dap.rs` (framing، launch/setBreakpoints/continue/next/stepIn، زنجیره stopped→threads→stack→scopes→vars) + پنل دیباگ (threads/stack/vars/output، فوروارد برک‌پوینت‌ها)
- یونیت‌تست: dap/lsp/code_editor/plugins/app/provider (روی ماشین کاربر اجرا می‌شوند؛ اینجا فقط کامپایل‌چک به‌خاطر killer محیط)
`cargo check -j 4` و `cargo check --tests`: سبز (فقط 2 وارنینگ deprecated).
انجام‌شده:
- `services/lsp/mod.rs`: `LspManager` (نخ پس‌زمینه + ران‌تایم tokio، open/change/complete/hover، ایونت diagnostics/completion/hover/status)
- `app.rs`: وایر LSP (diagnostics→گاتر، Ctrl+Space completion با غنی‌سازی سرور، Ctrl+K hover، پالت)، استتوس‌بار زنده
- `editor/`: `cursor_lsp_pos` (UTF-16)، `jump_to_line`، `diagnostic_counts`، `set_suggestions/is_visible`، مینی‌مپ واقعی (bars + خط کرسر + کلیک‌پرش)
- `build/runner.rs`: `AsyncRunner` واقعی (threads + drain + try_exit + stop)؛ `ui/build_panel.rs`: جاب نخ‌دار استریمینگ + Stop
- تست محیط: `cargo test` (codegen) روی این باکس کشته می‌شود (حتی `syn` با `-j1`)؛ hello-world اوکی — محدودیت محیط، نه کد.
`cargo check -j 4`: سبز (EXIT:0، فقط 2 وارنینگ deprecated root_path/root_uri).
فیکس‌های لوکال (12 فایل، پوش نشده):
- `services/terminal/shell.rs`: تک `openpty` صحیح (حذف double-pair)، `master` غیر-mut
- `languages/` (docker,kotlin,pascal,shell,swift,web): برگرداندن `fn hl` داخل `impl` (بستن درست)
- `editor/code_editor.rs`: `language_php→language`، غیرفعال‌کردن `html` (ناسازگاری tree-sitter 0.22 vs 0.20)، `available.height()`، `allocate_ui_at_rect` به‌جای `put`، `TextEdit.show` برای `cursor_range` (char→byte)، `compute_syntax_highlights &mut` + clone، فیکس `font_id` move و `layouter mut`
- `services/lsp/client.rs`: `TextDocumentSyncClientCapabilities` + `work_done_progress_params`
- `ui/pty_terminal.rs`: `poll` بدون borrow-conflict (collect بعد parse)
- `app.rs`: `minimap_enabled`
- `build/debugger.rs`: `mut out` + `_line`
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
