# dotman

`dotman` is a simple yet powerful command-line tool for managing your dotfiles across multiple systems. 
It helps you track, synchronize, and manage configuration files easily through a centralized repository.

## Why dotman?

Managing configuration files (dotfiles) across multiple machines can be challenging. `dotman` solves this by:

- Creating a central repository for all your dotfiles
- Maintaining symlinks to their original locations
- Making it easy to set up your environment on a new system
- Providing simple commands to track, update, and restore your configurations

## Installation

```bash
# Installation instructions coming soon
```

## Quick Start

```bash
# Initialize a new dotman project in the current directory
dotman init .

# Add existing dotfiles to your project
dotman add ~/.bashrc
dotman add ~/.config/nvim

# Check the status of your dotfiles
dotman status

# Set up dotfiles on a new system
dotman setup

# Restore original files
dotman restore
```

## How It Works

1. **Initialize a repository**: `dotman init .` creates a `.dotman.toml` configuration file in the current directory, establishing it as a dotman project.

2. **Add dotfiles**: When you run `dotman add ~/.bashrc`, dotman:
   - Moves the original file into your dotman project
   - Creates a symlink from the original location to the file in your project
   - Records this relationship in `.dotman.toml`

3. **Track status**: Use `dotman status` to see which dotfiles are being managed and their current state.

4. **Setup on new systems**: After cloning your dotman project to a new system, run `dotman setup` to create all the necessary symlinks based on the information in `.dotman.toml`.

5. **Restore when needed**: If you want to revert back to regular files (removing symlinks), use `dotman restore`.

## Commands

### `dotman init [directory]`

Initializes a new dotman project in the specified directory (defaults to current directory if not specified).

```bash
dotman init .
dotman init ~/dotfiles
```

### `dotman add <file_or_directory>`

Adds a dotfile or directory to your dotman project.

```bash
dotman add ~/.bashrc
dotman add ~/.config/nvim
dotman add ~/.gitconfig
```

### `dotman status`

Shows the current status of all managed dotfiles, including any that might be out of sync.

```bash
dotman status
```

### `dotman setup`

Creates symlinks for all dotfiles in your project based on the `.dotman.toml` configuration.

```bash
dotman setup
```

### `dotman restore`

Removes symlinks and restores original files to their locations.

```bash
dotman restore
```

## Example Workflow

1. **Initial setup on your main machine**:
   ```bash
   # Create a dotfiles repository
   mkdir ~/dotfiles
   cd ~/dotfiles
   dotman init .
   
   # Add your important dotfiles
   dotman add ~/.bashrc
   dotman add ~/.vimrc
   dotman add ~/.gitconfig
   dotman add ~/.config/alacritty
   
   # Commit to version control (optional but recommended)
   git init
   git add .
   git commit -m "Initial dotfiles setup"
   git remote add origin your-git-repo-url
   git push -u origin main
   ```

2. **Setting up on a new machine**:
   ```bash
   # Clone your dotfiles repository
   git clone your-git-repo-url ~/dotfiles
   cd ~/dotfiles
   
   # Create all symlinks
   dotman setup
   ```

## Tips

- Group unrelated configurations into separate dotman project
- Run `dotman status` periodically to ensure your symlinks are intact

## License

[ LICENSE](LICENSE)

