



📦 Archive-Minter

A minimal, offline-first web article archiver built for speed and simplicity. Keep your favorite articles, tutorials, and reads safe, readable, and accessible forever—even without an internet connection.
As a student developer, I created Archive-Minter to solve the problem of "link rot" and to provide a more sustainable way to manage the information I need for my studies. Whether you are archiving academic papers, technical documentation, or long-form essays, Archive-Minter keeps your knowledge organized, permanent, and ready for whenever you need it.

✨ Features

-Offline-First: Save an article once, read it anywhere, forever. No broken links or paywalls down the line.

-Lightweight & Fast: Built with a Rust backend (Tauri) and a Svelte frontend, ensuring minimal RAM usage and rapid load times.

-Distraction-Free: Strips away ads, pop-ups, and clutter, leaving only the pure text and images you actually want to read.

-Native Linux Support: Compiled and tested for Linux desktop environments.

🛠️ Tech Stack

-Frontend: SvelteKit, HTML/CSS/JS

-Backend: Rust, Tauri

-Build Target: Linux (Windows and MacOS coming very soon)

🚀 Installation & Quick Start

From the Pre-compiled Binary (Gumroad / GitHub Releases):

1. Download the Archive-Minter-Linux.zip or .tar.gz file.

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

From Source:
If you prefer to build it yourself, ensure you have Node.js and the Rust toolchain installed.

    git clone https://github.com/DionisGedeshi/archive-minter.git
    cd archive-minter
    npm install
    npm run tauri build

🤝 Contributing

Feel free to dive into the source code, submit issues, or create pull requests. Since this was built as a passion project, any feedback or optimizations are highly appreciated!

📜 License

This project is open-source and available under the MIT License.
