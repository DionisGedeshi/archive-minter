## 📦 Archive-Minter

A minimal, offline-first web article archiver built for speed and simplicity. Keep your favorite articles, tutorials, and reads safe, readable, and accessible forever—even without an internet connection.
As a student developer, I created Archive-Minter to solve the problem of "link rot" and to provide a more sustainable way to manage the information I need for my studies. Whether you are archiving academic papers, technical documentation, or long-form essays, Archive-Minter keeps your knowledge organized, permanent, and ready for whenever you need it.

## ✨ Features

**-Offline-First:** Save an article once, read it anywhere, forever. No broken links or paywalls down the line.

**-Lightweight & Fast:** Built with a Rust backend (Tauri) and a Svelte frontend, ensuring minimal RAM usage and rapid load times.

**-Distraction-Free:** Strips away ads, pop-ups, and clutter, leaving only the pure text and images you actually want to read.

**-Native Linux Support:** Compiled and tested for Linux desktop environments.

## 🛠️ Tech Stack

**-Frontend:** SvelteKit, HTML/CSS/JS

**-Backend:** Rust, Tauri

**-Build Target:** Linux (Windows and MacOS coming very soon)

## 🚀 Installation & Quick Start

You can download the free, pre-compiled installers for your operating system via [Gumroad](https://dionisg.gumroad.com/l/hnqqrp).

### 🪟 Windows
1. Download either the `.msi` (recommended) or the `.exe` setup file.
2. Double-click the file to run the installer.
3. **Important Note:** Because the executable is newly compiled and unsigned, Windows SmartScreen may show a blue popup saying "Windows protected your PC." 
   * Click **More info**.
   * Click **Run anyway**.

### 🍎 macOS (Universal: Apple Silicon & Intel)
1. Download the `Archive-Minter_universal.dmg` file.
2. Double-click to open the `.dmg` and drag the **Archive-Minter** app into your `Applications` folder.
3. **Important Note:** Because this is an indie open-source app, Apple's Gatekeeper will flag it as coming from an "unidentified developer." To open it for the first time:
   * Go to your `Applications` folder.
   * **Right-click** (or `Control` + click) the Archive-Minter app and select **Open**.
   * Click **Open** again in the warning dialog box to confirm.
  
### 🐧 Linux

1. Download either the archive-minter-linux.zip or archive-minter-linux.tar.gz file.

2. Extract the archive to your preferred directory.

3. Open your terminal and navigate to the extracted folder:

    ```
    cd path/to/Archive-Minter-Linux

4. Make the binary executable (if it isn't already):

    ```
    chmod +x archive-minter

5. Run the app:

    ```
    ./archive-minter

### From Source:
If you prefer to build it yourself, ensure you have Node.js and the Rust toolchain installed.

    git clone https://github.com/DionisGedeshi/archive-minter.git
    cd archive-minter
    npm install
    npm run tauri build

## 🤝 Contributing

Feel free to dive into the source code, submit issues, or create pull requests. Since this was built as a passion project, any feedback or optimizations are highly appreciated!

## 📜 License

This project is open-source and available under the MIT License.
