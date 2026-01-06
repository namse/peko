#!/bin/bash

echo "Fixing workspace permissions..."
sudo chown -R $(whoami) /workspace

echo "Configuring git safe directory..."
sudo git config --system --add safe.directory /workspace

if [ -d "/mnt/host-gh" ]; then
    echo "Setting up GitHub CLI..."
    mkdir -p $HOME/.config/gh
    
    sudo cp -r /mnt/host-gh/* $HOME/.config/gh/
    
    sudo chown -R $(whoami) $HOME/.config/gh
    
    chmod -R 700 $HOME/.config/gh
    
    gh auth setup-git
    echo "GitHub CLI setup complete."
else
    echo "Warning: /mnt/host-gh not found. Skipping GitHub CLI setup."
fi
