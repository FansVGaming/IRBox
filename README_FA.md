<div align="center">

# 🌐 IRBox Client

![IRBox Screenshot](screenshot.png)

**اپلیکیشن IRBox یک کلاینت پروکسی انعطاف‌پذیر و امن است که با فناوری‌های مدرن ساخته شده تا اتصال اینترنتی بی‌دردسر و قابل اعتماد را فراهم کند**

این نرم‌افزار برای کاربران آگاه از حریم خصوصی طراحی شده و از پشتیبانی چند پروتکلی، قابلیت‌های مسیریابی پیشرفته و ابزارهای مدیریتی ساده برخوردار است تا تجربه مرور امن و بدون مشکلی را تضمین کند.

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE) 
[![Releases](https://img.shields.io/github/downloads/frank-vpl/IRBox/total.svg)](https://github.com/frank-vpl/IRBox/releases/latest)
[![Latest Release](https://img.shields.io/github/v/release/frank-vpl/IRBox)](https://github.com/frank-vpl/IRBox/releases/latest)

[English Version](README.md)

</div>

## 🚀 ویژگی‌های کلیدی

### پشتیبانی چند پروتکلی
- **VLESS**
- **VMess**
- **Shadowsocks**
- **Trojan**
- **Hysteria2**
- **TUIC**
- **SSH**

### مدیریت پیشرفته
- **پشتیبانی از اشتراک** - درون‌ریزی و به‌روزرسانی خودکار لینک‌های اشتراک
- **قوانین مسیریابی** - قوانین مبتنی بر دامنه (پروکسی/مستقیم/مسدود) با پیش‌تنظیماتی برای مسدودسازی تبلیغات و دور زدن منطقه‌ای
- **تونل‌زنی تقسیم** - انتخاب مسیر پیش‌فرض: تمام ترافیک یا دامنه‌های انتخابی را پروکسی کنید

### حالت‌های اتصال
- **پروکسی سیستم** - پروکسی HTTP برای دسترسی سراسر سیستم
- **حالت TUN** - VPN کامل که تمام ترافیک را ضبط می‌کند
- **ارتقاء مدیر** - "اجرای با عنوان مدیر" با یک کلیک برای حالت TUN

### تجربه کاربری
- **آشنایی اولیه** - تور تعاملی راهنما برای کاربران جدید
- **پینگ TCP** - تست تاخیر انبوه سرورها
- **انتخاب خودکار بهترین سرور** - انتخاب هوشمند سرور
- **تم‌ها** - ۲ تم رنگی (تیره، روشن)
- **سبک‌ها** - پیش‌فرض، حداقلی

## 🛠️ نصب

### پیش‌نیازها
- Rust و Cargo
- Tauri CLI
- NodeJS و NPM 
- پیش‌نیازهای Tauri

### راه‌اندازی سریع

1. **کلون کردن مخزن**
   ```bash
   git clone https://github.com/frank-vpl/IRBox.git
   cd IRBox
   ```

2. **نصب وابستگی‌ها**
   ```bash
   npm install
   ```
   
3. **نصب Tauri CLI**
   ```bash
   cargo install tauri-cli --version ^2
   ```

4. **دانلود هسته‌ها**

   **ویندوز:**
   ```bash
   ./cores.bat
   ```
   
   **لینوکس/مک:**
   ```bash
   chmod +x cores.sh
   ./cores.sh
   ```

## 🚀 استفاده

### توسعه
```bash
cargo tauri dev
```

### تولید
```bash
cargo tauri build
```

## 🤝 مشارکت

مشارکت‌ها خوش‌آمد هستند! لطفاً راحت باشید و یک درخواست کشش (Pull Request) ارسال کنید. برای تغییرات عمده، لطفاً ابتدا یک موضوع (issue) باز کنید تا در مورد آنچه می‌خواهید تغییر دهید، بحث کنیم.

## 📄 مجوز

این پروژه تحت مجوز عمومی گنو نسخه ۳.۰ (GPL-3.0) مجوز داده شده است - برای جزئیات بیشتر فایل [LICENSE](LICENSE) را ببینید.

### فناوری‌های هسته‌ای

اپلیکیشن IRBox از دو فناوری پیشرو در زمینه پروکسی استفاده می‌کند:

<div align="center">

| هسته | توضیحات |
|------|---------|
| [Xray-core](https://github.com/XTLS/Xray-core) | یک پلتفرم برای ساخت پروکسی‌های دور زدن محدودیت‌های شبکه |
| [sing-box](https://github.com/SagerNet/sing-box) | پلتفرم جهانی پروکسی |

</div>

### مجوزهای کتابخانه‌های شخص ثالث

- [Rust](https://www.rust-lang.org/) - [مجوز](./licenses/rust.md)
- [Tauri](https://v2.tauri.app/) - [مجوز](./licenses/tauri.md)
- [sing-box](https://github.com/SagerNet/sing-box) - [مجوز](./licenses/sing-box.md)
- [Xray-core](https://github.com/XTLS/Xray-core) - [مجوز](./licenses/xray.md)

## 🙏 قدردانی

- ساخته شده با [Tauri](https://tauri.app/) - چارچوبی برای ساخت برنامه‌های محلی امن
- قدرت گرفته از [sing-box](https://github.com/SagerNet/sing-box) و [Xray-core](https://github.com/XTLS/Xray-core)
- الهام گرفته از نیاز به راه‌حل‌های VPN امن و انعطاف‌پذیر

## 📚 مستندات
[مستندات IRBox](./docs/README.md)

## 🎨 دارایی‌های طراحی

<div align="center">

### لوگو و آیکون‌های برنامه
![PiraIcons](https://img.shields.io/badge/Icons_by-Hossein_Pira-3d85c6?style=for-the-badge&logo=github)

- آیکون‌ها توسط حسین پیرا – [PiraIcons](https://github.com/code3-dev/piraicons-assets) - [مجوز](./licenses/piraicons.md)

</div>

## 🧩 فناوری‌های مورد استفاده

<div align="center">

### وابستگی‌های ظاهری
![React](https://img.shields.io/badge/React-20232a?style=for-the-badge&logo=react&logoColor=61DAFB)
![TypeScript](https://img.shields.io/badge/TypeScript-007ACC?style=for-the-badge&logo=typescript&logoColor=white)
![Vite](https://img.shields.io/badge/Vite-B73BFE?style=for-the-badge&logo=vite&logoColor=FFD62E)

### چارچوب و هسته
![Tauri](https://img.shields.io/badge/Tauri-FFD62E?style=for-the-badge&logo=tauri&logoColor=black)
![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)

</div>

### وابستگی‌ها
- [react](https://react.dev/) - یک کتابخانه جاوا اسکریپت برای ساخت رابط‌های کاربری
- [react-dom](https://reactjs.org/docs/react-dom.html) - متدهای خاص DOM را فراهم می‌کند که می‌توانند در سطح بالای برنامه شما استفاده شوند
- [@tauri-apps/api](https://github.com/tauri-apps/tauri) - اتصالات API Tauri
- [@tauri-apps/plugin-deep-link](https://github.com/tauri-apps/plugins-workspace) - افزونه Tauri برای پیوند عمیق
- [@tauri-apps/plugin-shell](https://github.com/tauri-apps/plugins-workspace) - افزونه Tauri برای عملیات پوسته

#### وابستگی‌های توسعه
- [typescript](https://www.typescriptlang.org/) - تایپ‌اسکریپت یک زیرمجموعه تایپ‌دار از جاوا اسکریپت است که به جاوا اسکریپت ساده کامپایل می‌شود
- [vite](https://vitejs.dev/) - ابزارآلات ظاهری نسل بعدی
- [@vitejs/plugin-react](https://github.com/vitejs/vite-plugin-react) - افزونه Vite برای پروژه‌های React
- [@tauri-apps/cli](https://github.com/tauri-apps/tauri) - رابط خط فرمان Tauri
- [@types/react](https://www.npmjs.com/package/@types/react) - تعاریف تایپ برای React
- [@types/react-dom](https://www.npmjs.com/package/@types/react-dom) - تعاریف تایپ برای ReactDOM