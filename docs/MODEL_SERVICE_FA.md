# سرویس مدیریت مدل‌ها و مدل‌سرور محلی (فارسی)

این سند شرح می‌دهد که چگونه "مدل‌ها" در پروژهٔ SuperAgentCli مدیریت شده و چگونه سرویس مدل محلی (Local Model Server) کار می‌کند.

هدف این سرویس: فراهم‌کردن API مشابه OpenAI برای مدل‌های محلی، پوشش‌دهی llama.cpp و ollama در حالت اتوماتیک، و فراهم آوردن یک Provider مستقل (Mock و قابل توسعه) که در صورت نبود llama.cpp یا ollama امکان تست محلی را می‌دهد.


## اهداف طراحی

- پوشش‌دهی خودکار مدل‌ها در پوشهٔ مشخص (XDG): `~/.local/share/super-agent/models` به صورت پیش‌فرض.

- کشف (discover) خودکار مدل‌ها و نمایش در TUI و CLI.

- امکان واردسازی (import) مدل‌ها از هر مسیر با کلیک یا دستور CLI.

- سرویس HTTP داخلی که endpointهای `/v1/models` و `/v1/chat/completions` را ارائه می‌دهد.

- Providerها: LlamaCpp (wrapper), Ollama (wrapper), Mock (درون‌ساخت). در v0.1 Mock کامل است، wrapperهای Llama/Ollama اسکلت دارند و در نسخه‌های بعدی توسعه می‌یابند.

- مدیریت providerها: امکان register provider برای یک مدل و استفاده از آن از طریق TUI/CLI.

## فایل‌ها و ماژول‌ها
- `src/models/manager.rs` — کشف/واردسازی/حذف مدل‌ها.
- `src/models/server.rs` — سرویس مدل محلی، Provider trait و یک `MockProvider` و تابعی برای راه‌اندازی یک HTTP server با `axum`.
- `src/config.rs` — تنظیم `model_dir` و `model_server_addr` و ذخیرهٔ آن.
- CLI: `agent models list|import|remove|serve start <model>`.
- TUI: پنل مدیریت مدل‌ها (`m` کلید) و واردسازی از داخل TUI (`i` کلید).

## چگونگی استفاده
- لیست مدل‌ها:
  - CLI: `agent models list`
  - TUI: داخل برنامه فشار `m`
- واردسازی مدل:
  - CLI: `agent models import /path/to/model.gguf`
  - TUI: `m` سپس `i` سپس مسیریابی و Enter
- شروع سرویس مدل محلی (mock provider برای مدل):
  - CLI: `agent models serve start mymodel`
  - سپس می‌توانید با ابزارهای HTTP به `http://127.0.0.1:11400/v1/chat/completions` درخواست بفرستید.

## توسعهٔ بعدی
- پیاده‌سازی کامل Provider برای llama.cpp (پیدا کردن باینری، راه‌اندازی، صحّه‌گذاری، proxy requests).
- پیاده‌سازی Provider برای Ollama مشابه بالا.
- ساخت Provider "native" برای اجرای مستقیم مدل‌ها (FFI/Bindings) تا وابستگی به باینری خارجی لازم نباشد — این همان هدف شما برای داشتن سرویس مستقل و قابل نصب است.
- اضافه کردن authentication / token management برای سرویس (شبیه OpenAI API keys).

## راهنمای فنی دقیق‌تر (چگونه provider واقعی بسازیم و اتوماسیون کشف/نصب را پیاده کنیم)
1) معماری Provider
- هر Provider باید `Provider` trait را پیاده‌سازی کند و متدهای async زیر را داشته باشد:
  - `start()` — راه‌اندازی سرویس یا فرایند زیرساختی (مثلاً اجرای باینری llama.cpp با آرگومان‌های لازم).
  - `stop()` — توقف سرویس و پاکسازی منابع.
  - `is_running()` — چک سلامت (مثلاً تماس HTTP به health endpoint).
  - `chat(messages)` — ارسال پیام‌ها به مدل و بازگرداندن پاسخ متنی.

2) اتوماسیون تشخیص و نصب
- تشخیص باینری:
  - در مسیرهای شناخته‌شده (`$PATH`, `/usr/local/bin`, `~/.local/bin`) به دنبال `server` یا نام باینری llama.cpp یا ollama بگردید.
- در صورت نبود باینری:
  - پیشنهاد به کاربر برای دانلود یا اجرای اسکریپت نصب. (مستندات نصب باید سیستم‌عامل-محور باشد)
  - امکان اجرای یک helper script از داخل پروژه برای دانلود یا build (در صورتی که کاربر رضایت دهد).

3) مدیریت مدل‌ها
- پوشهٔ مدل‌ها (`model_dir`) محل قرارگیری فایل‌هاست. فرمت‌ها: `gguf`, `safetensors`, `pth`, `pt`, `bin`, و غیره — تشخیص اولیه بر مبنای پسوند است.
- metadata: در آینده برای هر مدل یک فایل `metadata.toml` ساخته می‌شود تا اطلاعاتی مانند tokenizer type، quantization، recommended provider و موارد مربوطه ذخیره شود.

4) API شبیه OpenAI
- Endpointها (تطابق با OpenAI minimal):
  - `GET /v1/models` => لیست مدل‌های شناخته‌شده
  - `POST /v1/chat/completions` => ارسال messages و دریافت choices
- auth: در آینده افزوده خواهد شد؛ mechanism پیشنهادی: token-based header `Authorization: Bearer <token>`.

5) پشتیبانی از اجرای مدل بدون llama.cpp/ollama (طراحی یک Provider native)
- این کار نیازمند binding یا FFI به کتابخانه‌های inference (مثل ggml یا bindings زبان Rust به libggml).
- راه‌حل تدریجی: ابتدا یک Provider native با استفاده از crates موجود یا با لایهٔ C FFI نوشته شود. این Provider می‌تواند مدل‌های کوچک را در CPU بارگذاری کند و برای GPU نیاز به پشتیبانی CUDA/VULKAN دارد که پیچیدگی بیشتری دارد.

## نکات عملیاتی (توصیه‌ها)
- v0.1: از MockProvider برای تست UI و flows استفاده کنید.
- برای محیط Termux: ساخت باینری‌های native ممکن است دشوار باشد؛ بهترین راه برای شروع استفاده از سروری بر پایه‌ی ماشین میزبان یا build toolchain مخصوص Android است.

## مثال‌ها
- راه‌اندازی سرویس نمونه برای مدل `mymodel` (CLI):
  1. `agent models import /path/to/mymodel.gguf`
  2. `agent models serve start mymodel`
  3. سپس در برنامه یا curl به `http://127.0.0.1:11400/v1/chat/completions` درخواست بفرستید.

---

اگر می‌خواهید، می‌توانم برای مرحلهٔ بعدی یکی از گزینه‌های زیر را انجام دهم:
- A) پیاده‌سازی سازندهٔ `LlamaProvider` که به طور خودکار باینری llama.cpp را در سیستم جستجو کرده و در صورت وجود آن را اجرا و health-check کند. ✔️
- B) شروع روی `NativeProvider` با بررسی crates موجود (gguf/ggml bindings) و نوشتن نمونهٔ اولیۀ بارگذاری مدلِ GGUF و پاسخ‌دهی ساده. ✔️

کدام گزینه را اول پیش ببرم؟

---

اگر مایل باشید، در مرحلهٔ بعد من:
- wrapper واقعی برای llama.cpp را پیاده‌سازی می‌کنم که باینری را خودکار جستجو کرده، در صورت عدم وجود روند نصب/دانلود را راهنمایی کند، راه‌اندازی و مدیریت آن را انجام دهد.
- یا یک Provider native (ابداعی) شروع می‌کنم که با FFI یا bindingهای موجود مدل‌های GGUF را بارگذاری کند (نیاز به کتابخانه‌های native و تلاش بیشتر).

کدام گزینه را در مرحلهٔ بعد ترجیح می‌دهید؟
