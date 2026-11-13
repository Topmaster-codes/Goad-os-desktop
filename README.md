# Goad-OS Linux

<img width="1920" height="1080" alt="Aura" src="https://github.com/user-attachments/assets/477a67a4-7980-4d59-8e80-c17c04e1e3a8" />

## PLEASE if you have any ideas for new logo designs that you would like to donate TELL ME, I whipped ts up in canva in like three seconds (and now it wont load in github)

### A Rust-powered desktop. Built on Hyprland. Powered by CachyOS. 

**Goad-OS** is an experimental, CachyOS-based Linux distribution built on the Arch Linux ecosystem built on another experimental project (Amane). It combines the flexibility and performance of an Arch-based system with a carefully designed desktop experience powered by **Hyprland, Amane, Rust, and Rofi** with a soon to be coming **Siri like local assistant**. With an emphasis on performance using the linux-cachyos-bore-lto

This repository right now is nothing more than a collection of configurations, Goad-OS will an attempt to create a distinctive Linux desktop with its own identity, workflow, and philosophy. (Type shit twin!)

The project focuses on delivering a cohesive, ready-to-use desktop environment while exploring new ways to interact with Linux through custom software, modern desktop technologies (larping rust), and a local AI-powered assistant (thats fancy for inputting commands through voice recognition).

Currently in **BETA**, Goad-OS will be actively evolving.

---

## What Is Goad-OS?

Goad-OS to me is fixing everything I don't like about hyprland configurations today, and how I hate how slow quickshell can be.

Its foundation consists of three primary projects:

- **Hyprland** — Provides the Wayland compositor, window management, and visual effects.
- **Amane** — Powers the custom Rust-based desktop shell and interface.
- **Rofi** — Provides application launching, menus, and additional desktop interactions.

I aim to create an OS as a sort of Omarchy alternet without all the bloat, and most importantly RUST

My main goal is simple: create a Linux desktop that feels like a cohesive operating system rather than a collection of independently configured applications like ive seen most hyprland setups do.

## Features & Highlights

### Rust-Powered Desktop

Goad-OS uses **Amane**, an experimental Rust-based framework for building Wayland desktop shells.

The custom desktop interface is an important part of the project's identity. Rather than relying entirely on traditional desktop environments, Goad-OS explores a more customized approach to the Linux desktop using Rust.

### Hyprland at the Core

[Hyprland](https://hypr.land/) serves as the underlying Wayland compositor.

Its dynamic tiling, animations, and extensive customization capabilities provide the foundation for the GoaDOS desktop experience.

### Rofi Integration

[Rofi](https://github.com/davatorium/rofi) provides application launching, menus, and additional desktop interactions.

It complements the Amane desktop shell and helps create a flexible, keyboard-friendly workflow.

### CachyOS Foundation

GoaDOS is built on the CachyOS ecosystem and its Arch Linux foundation.

This provides a flexible starting point with access to the broader Arch Linux software ecosystem, pacman package management, and a highly customizable Linux environment.

### AI Assistant — A Smarter Desktop

One of the longer-term goals for GoaDOS is to introduce a custom **LOCAL** AI-powered assistant because Sam Altman wants your data and ram.

Inspired by conversational assistants such as Siri, the idea is to make interacting with Linux feel more natural through voice and intelligent assistance.

The planned assistant is intended to be optional and just a gimmick, I would never force something on the user (that is hella illegal btw).

Assistant functionality is part of the project's development vision and is not available in the current beta.

### A Desktop With Its Own Identity

Goad-OS is being developed around a **consistent** visual identity and desktop workflow.

The interface, menus, system configurations, utilities, and planned assistant are all part of a larger effort to create an operating system that feels intentionally designed and themed, because apparently im the only one that likes high contrast blues.

---

## Technology Stack

| Component | Technology |
|---|---|
| Distribution foundation | CachyOS |
| Linux ecosystem | Arch Linux |
| Window compositor | Hyprland |
| Display protocol | Wayland |
| Desktop shell framework | Amane |
| Desktop shell language | Rust |
| Menu system | Rofi |
| Package management | Pacman |
| Software ecosystem | Arch-compatible Linux packages |

### Core Technologies & Projects

- [CachyOS](https://cachyos.org/)
- [Arch Linux](https://archlinux.org/)
- [Hyprland](https://hypr.land/)
- [Amane](https://github.com/MystiaFin/amane)
- [Rust](https://www.rust-lang.org/)
- [Rofi](https://github.com/davatorium/rofi)

---

## Project Status

**BETA DEVELOPMENT**

Goad-OS is currently in active beta development.

The primary focus is establishing a stable foundation, refining the desktop experience, improving system integration, and developing the features that will define future releases.

As an experimental Desktop, Goad-OS may contain:

- Incomplete or experimental functionality.
- Desktop interface bugs.
- Inconsistent styling.
- Unexpected behavior.
- HELLA bugs.
- Configuration and compatibility issues.
- Features that change between releases.

Not all planned features are necessarily implemented or available.

Testing, feedback, and contributions are welcome.

---

## Screenshots & Demonstrations

Screenshots and demonstrations will be added as the project develops (and when I get home).

<!-- Add screenshots here when available.

![GoaDOS Desktop Overview](docs/screenshots/desktop.png)

![GoaDOS Desktop Shell](docs/screenshots/shell.png)

![GoaDOS Rofi Menus](docs/screenshots/menus.png)

-->

---

## Installation

This repository currently provides the desktop configuration and installer, not a complete bootable Goad-OS ISO. The installer builds the Rust shell from the included source and installs user-level configuration.

See **[Install instructions](Install%20instructions.md)** for prerequisites, bundling, installation, and backup behavior.

From a Git checkout:

```sh
./bundle.sh
```

Copy the resulting `goad-os-amane.tar.gz` to the target machine, extract it, enter the extracted project directory, and run `./install.sh`. Review the installation guide before running it; the project is experimental and currently targets a compatible CachyOS/Arch + Hyprland setup.

No official bootable ISO or tagged binary release is published by this repository at this time.

---

## Development Roadmap

Goad-os as ive been repeating is an evolving project. The following areas represent current development priorities and long-term goals.

### Desktop Experience
- [ ] Establish a stable beta foundation.
- [ ] Improve Amane desktop shell integration.
- [ ] Refine the Rust-powered desktop interface.
- [ ] Improve Rofi menu integration and consistency.
- [ ] Refine the Hyprland desktop experience.
- [ ] Develop a cohesive visual identity.

### System Integration
- [ ] Improve system configuration and usability.
- [ ] Establish reliable default configurations.
- [ ] Improve hardware compatibility.
- [ ] Refine package and application integration.
- [ ] Improve the installation experience.

### AI & Intelligent Features
- [ ] Develop the GoaDOS desktop assistant.
- [ ] Explore voice-based desktop interaction.
- [ ] Integrate intelligent system assistance.

### Community & Release
- [ ] Expand project documentation.
- [ ] Improve contributor documentation.
- [ ] Establish a consistent release process.
- [ ] Work toward a stable public release.

**Note:** Roadmap items represent development goals. They are not guarantees of availability in any particular release.

---

## Bug Reports & Feedback

Found a bug? Have an idea for a new feature? Want to suggest an improvement?

Feedback and bug reports are appreciated.

Please submit issues through the project's GitHub repository.

When reporting a bug, include as much of the following information as possible:

1. A clear description of the issue.
2. Steps to reproduce the problem.
3. Expected behavior.
4. Actual behavior.
5. Relevant terminal output or logs.
6. Hardware and GPU information.
7. The Goad-OS desktop version you are running.

Screenshots and recordings are especially helpful for visual issues.

---

## Contributing

Goad-OS is an open-source project built around the idea of creating a different kind of Linux desktop experience.

Contributions from developers, designers, Linux enthusiasts, and testers are welcome.

Whether you're interested in:

- Rust development.
- Linux distributions.
- Wayland and Hyprland.
- Desktop interface design.
- System configuration.
- AI and voice assistants.
- Testing and documentation.

Your contributions can help shape the future of Goad-OS.

For significant changes, please open an issue or discussion before submitting a pull request.

---

## Acknowledgments

Goad-OS is a sort of larp project that builds upon the work of several excellent open-source projects and communities.

Special thanks to:

- [CachyOS](https://cachyos.org/) — Distribution foundation.
- [Arch Linux](https://archlinux.org/) — Underlying Linux ecosystem.
- [Hyprland](https://hypr.land/) — Wayland compositor.
- [Amane](https://github.com/MystiaFin/amane) — Rust-based Wayland desktop shell framework.
- [Rofi](https://github.com/davatorium/rofi) — Application launcher and menu system.
- [Rust](https://www.rust-lang.org/) — Systems programming language.

Goad-OS is an independent project and is not affiliated with or officially endorsed by CachyOS, Arch Linux, Hyprland, Amane, or Rofi. (yet)

All third-party software remains subject to its respective license and ownership.

---

## License & Open Source

Goad-OS is open source. Anyone may use, study, modify, and redistribute the code and assets in this repository. The only thing reserved is the Goad-OS branding (see below).

How the repository is licensed:

- **Code and assets** (the Rust shell, installer and scripts, Hyprland/Lua configuration, and the themes and configs for kitty, rofi, mako, Qt, hyprfm, and fastfetch): GNU General Public License v3.0 or later (GPL-3.0-or-later). See [LICENSE](LICENSE).
- **Documentation** (this README and the install instructions): [Creative Commons Attribution-ShareAlike 4.0 (CC BY-SA 4.0)](https://creativecommons.org/licenses/by-sa/4.0/).
- **Third-party components:** Their respective original licenses. The bundled [Amane](https://github.com/MystiaFin/amane) library in `vendor/amane` is MIT licensed; its license is in [vendor/amane/LICENSE](vendor/amane/LICENSE). Rust dependencies keep their own licenses.

### Goad-OS Name & Branding

The Goad-OS name, logo, and official visual identity are separate from the software's open-source license. See [BRANDING.md](BRANDING.md) for what that means in practice.

Everyone is welcome to fork, modify, redistribute, and build upon Goad-OS's source code and assets under the licenses above.

However, modified distributions should use their own names, logos, and branding rather than presenting themselves as the official Goad-OS project.

Open-source software does not automatically grant permission to use another project's trademarks or imply official endorsement.

---

<div align="center">

# GOAD-OS

### Built with Rust. Powered by CachyOS. Driven by Hyprland.

**A different approach to the Linux desktop.**

*GoaDOS Linux — Currently in Beta.*

</div>
