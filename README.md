# Vocal Sample Pack Fixer

## Overview

**Vocal Sample Pack Fixer** is a simple, multi-threading audio batch processing tool written in Rust. Its primary function is to fix poorly edited voice sample packs by removing the silence at the start of each sample and adjusting the gain so the peak reaches 0.9. This tool is designed to handle large sets of audio files efficiently by utilizing all available CPU cores.

Currently, it supports `.wav` files, with plans to extend support for other file formats such as `.mp3` and `.flac`.

## Features

- **Silence Removal**: Automatically detects and removes long silences (with noise) at the beginning of each sample.
- **Gain Adjustment**: Adjusts the volume so the peak of the sample is normalized to 90% (0.9).
- **Multi-Threading**: Uses multi-threading to efficiently process large audio file collections, balancing the load across CPU cores.
- **Batch Processing**: Recursively scans directories and processes all audio files, creating an organized output structure.

## Requirements

- Rust (to build the project)
- Some samples you want to fix, needs to be `.wav` files

## Installation

1. Clone the repository:
    ```bash
    git clone https://github.com/ablackcat04/vocal_sample_pack_fixer.git
    cd vocal_sample_pack_fixer
    ```

2. Build the project:
    ```bash
    cargo build --release
    ```

3. Find the compiled executable in the `target/release/` folder.

If you need help installing cargo, you can refer the Rust Programming Book ch1-1. Heres the link: https://doc.rust-lang.org/stable/book/ch01-01-installation.html

## Usage

To use the tool, run the executable with the path to the folder containing the samples. Example:

```bash
./vocal_sample_pack_fixer.exe <path-to-your-sample-folder>
```

For example, if your sample folder is located at ../../samples, you would type:
```bash
./vocal_sample_pack_fixer.exe ../../samples
```

The program will process all the .wav files in the directory and its subdirectories, outputting the processed files in the outputs/ folder while preserving the relative directory structure.

## Future Enhancements

We are not actively working on improving the functionality of the audio batch processing tool. But there are some planned enhancements include:

- **Code Refactoring**
- **Customizability**: Customizable silence threshold and gain settings via command-line arguments.

- **Tail Cutting**: Implement a conservative approach to remove unwanted audio at the end of samples, ensuring that essential sounds are preserved.
- **Multi-Format Support**: Extend compatibility to various audio file types, including MP3, OGG, FLAC, and M4A, by integrating the **Symphonia** library.
- **Custom Output Destination**: Provide users the option to specify their desired output directory, rather than defaulting to `./outputs`.
- **Library Refactoring**: Split the batch processing logic into a dedicated library to facilitate code reuse and integration into other projects.
- **Dynamic Peak Volume Adjustment**: Develop a system to adjust the target peak volume based on audio content, allowing for softer peaks in quieter samples and potential compression for louder sounds.
- **Enhanced Silence Detection**: Explore advanced methods for detecting the start of audio samples, improving accuracy beyond the current silence detection implementation.
- **Advanced Loudness Measurement**: Introduce more sophisticated techniques for determining the output loudness, such as aligning with industry standards like **LUFS** (Loudness Units relative to Full Scale).

## Contributing

We welcome contributions to enhance the functionality and usability of the audio batch processing tool. Just open a pull requset. It's also welcome to repoet issues or suggestions via Github issues!

## License
This project is dual-licensed under both the MIT License and the Apache License (Version 2.0).
You may choose to use either license.
