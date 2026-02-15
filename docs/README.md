# IRBox Documentation

- Built for multiple platforms: **Windows**, **macOS**, and **Linux**
- Includes **sing-box** and **xray-core** executables
- Desktop application built with **Tauri**

---

## Assets

This release includes installers/packages for:

- **Windows (x86_64)**
- **macOS (Intel & Apple Silicon)**
- **Linux (x86_64 & ARM64)**

---

## System Requirements

- **Windows**: Windows 10 or later  
- **macOS**: macOS 10.15 or later  
- **Linux**: glibc 2.17 or later  

---

# Installing & Running IRBox on macOS

## 1. Install IRBox

1. Double-click the downloaded `.dmg` file.
2. Drag **IRBox** into the **Applications** folder.
3. Open it from **Applications** like any standard Mac app.

> ℹ️ IRBox starts in **Proxy Mode** by default — no additional permissions are required.

---

# Enabling TUN Mode (Routes All Traffic)

When switching to **TUN Mode**, IRBox requires elevated system privileges to manage network traffic.

You can enable it using one of the following methods:

---

## Method 1 — Grant Admin Access from Inside the App

1. Launch **IRBox**
2. Navigate to **Settings → VPN Mode**
3. Select **TUN**
4. Click **Run as Administrator**

---

## Method 2 — Start via Terminal

Run the following command:

```bash
sudo /Applications/IRBox.app/Contents/MacOS/IRBox
```