#!/usr/bin/env bash

# Check if sudo is available
if command -v sudo >/dev/null 2>&1; then
    PRIV_ESC="sudo"
# If sudo is not available, check if doas is available
elif command -v doas >/dev/null 2>&1; then
    PRIV_ESC="doas"
else
    echo "Error: Neither sudo nor doas is installed. Please install one of them."
    exit 1
fi

ACTION="install"

if [ "$1" = "--uninstall" ]; then
    ACTION="uninstall"
    shift
fi

BRANCH="${1:-stable}"

case "$BRANCH" in
    stable|canary|ptb)
        # Valid branch, do nothing
        ;;
    *)
        echo "Invalid branch: $BRANCH. Please choose 'stable', 'canary', or 'ptb'."
        exit 1
        ;;
esac

GITHUB_ORG="MeguminSama"
REPO_NAME="moonlight-launcher"

ASSET_NAME="moonlight-$BRANCH"

if [ "$ACTION" = "uninstall" ]; then
    echo "Uninstalling moonlight $BRANCH..."

    # Remove the binary
    $PRIV_ESC rm -f "/usr/bin/moonlight-$BRANCH"

    # Check if all three binaries are deleted
    if [ ! -f "/usr/bin/moonlight-stable" ] && \
       [ ! -f "/usr/bin/moonlight-canary" ] && \
       [ ! -f "/usr/bin/moonlight-ptb" ]; then
        # Remove the shared library if all binaries are gone
        $PRIV_ESC rm -f "/usr/lib/libmoonlight_launcher.so"
        $PRIV_ESC ldconfig /usr/lib
    fi

    # Remove icons
    HICOLOR="/usr/share/icons/hicolor"
    $PRIV_ESC rm -f "$HICOLOR/16x16/apps/moonlight-$BRANCH.png"
    $PRIV_ESC rm -f "$HICOLOR/32x32/apps/moonlight-$BRANCH.png"
    $PRIV_ESC rm -f "$HICOLOR/48x48/apps/moonlight-$BRANCH.png"
    $PRIV_ESC rm -f "$HICOLOR/64x64/apps/moonlight-$BRANCH.png"
    $PRIV_ESC rm -f "$HICOLOR/128x128/apps/moonlight-$BRANCH.png"
    $PRIV_ESC rm -f "$HICOLOR/256x256/apps/moonlight-$BRANCH.png"
    $PRIV_ESC rm -f "$HICOLOR/512x512/apps/moonlight-$BRANCH.png"
    $PRIV_ESC rm -f "$HICOLOR/1024x1024/apps/moonlight-$BRANCH.png"

    # Update icon cache
    $PRIV_ESC gtk-update-icon-cache /usr/share/icons/hicolor/ || true

    # Remove desktop entry
    if [ "$BRANCH" = "stable" ]; then
        DESKTOP_ENTRY_FILENAME="moonlight.desktop"
    else
        DESKTOP_ENTRY_FILENAME="moonlight-$BRANCH.desktop"
    fi

    $PRIV_ESC rm -f "/usr/share/applications/$DESKTOP_ENTRY_FILENAME"

    echo "Uninstallation complete!"
    exit 0
fi

# Convert BRANCH to uppercase or capitalize as needed
if [ "$BRANCH" = "ptb" ]; then
    BRANCH_UPPER="PTB"
else
    BRANCH_UPPER=$(echo "$BRANCH" | awk '{print toupper(substr($0,1,1)) substr($0,2)}')
fi

LATEST_RELEASE=$(curl -s "https://api.github.com/repos/$GITHUB_ORG/$REPO_NAME/releases/latest")

VERSION=$(echo "$LATEST_RELEASE" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$VERSION" ]; then
    echo "Failed to fetch the latest release. Please try again or open an issue on GitHub."
    exit 1
fi

ASSET_FILENAME="${ASSET_NAME}-${VERSION}.tar.gz"

echo "Downloading moonlight $BRANCH $VERSION ($ASSET_FILENAME) from GitHub..."

DOWNLOAD_URL=$(echo "$LATEST_RELEASE" | grep "browser_download_url" | grep "$ASSET_FILENAME" | cut -d '"' -f 4)

if [ -z "$DOWNLOAD_URL" ]; then
    echo "Failed to fetch the download URL. Please try again or open an issue on GitHub."
    exit 1
fi

TEMP_DIR=$(mktemp -d)

curl -L -o "$TEMP_DIR/$ASSET_FILENAME" "$DOWNLOAD_URL"

if [ $? -eq 0 ]; then
    echo "Download successful!"
else
    echo "Download failed. Please try again or open an issue on GitHub."
    exit 1
fi

tar -xzf "$TEMP_DIR/$ASSET_FILENAME" -C "$TEMP_DIR"

if [ $? -eq 0 ]; then
    echo "Extraction successful!"
else
    echo "Extraction failed. Please try again or open an issue on GitHub."
    rm -r "$TEMP_DIR"
    exit 1
fi

echo "Installing moonlight $BRANCH $VERSION..."

$PRIV_ESC install -Dm 755 "$TEMP_DIR/moonlight-$BRANCH" /usr/bin/

OTHER_BRANCHES=$(ls /usr/bin | grep 'moonlight-' | grep -v "moonlight-$BRANCH")

if [ -n "$OTHER_BRANCHES" ]; then
    # TODO: Maybe add a version flag to check if the other branches are outdated?
    echo "Warning: The following moonlight branches are also installed. If they are outdated, consider updating them too:"
    echo "$OTHER_BRANCHES"
fi

$PRIV_ESC install -Dm 644 "$TEMP_DIR/libmoonlight_launcher.so" /usr/lib/
$PRIV_ESC ldconfig /usr/lib

if [ "$BRANCH" = "stable" ]; then
    DESKTOP_ENTRY_NAME="moonlight"
    DESKTOP_ENTRY_FILENAME="moonlight.desktop"
else
    DESKTOP_ENTRY_NAME="moonlight $BRANCH_UPPER"
    DESKTOP_ENTRY_FILENAME="moonlight-$BRANCH.desktop"
fi

HICOLOR="/usr/share/icons/hicolor"

$PRIV_ESC install -Dm644 "$TEMP_DIR/icons/icon-16x16.png" "$HICOLOR/16x16/apps/moonlight-$BRANCH.png"
$PRIV_ESC install -Dm644 "$TEMP_DIR/icons/icon-32x32.png" "$HICOLOR/32x32/apps/moonlight-$BRANCH.png"
$PRIV_ESC install -Dm644 "$TEMP_DIR/icons/icon-48x48.png" "$HICOLOR/48x48/apps/moonlight-$BRANCH.png"
$PRIV_ESC install -Dm644 "$TEMP_DIR/icons/icon-64x64.png" "$HICOLOR/64x64/apps/moonlight-$BRANCH.png"
$PRIV_ESC install -Dm644 "$TEMP_DIR/icons/icon-128x128.png" "$HICOLOR/128x128/apps/moonlight-$BRANCH.png"
$PRIV_ESC install -Dm644 "$TEMP_DIR/icons/icon-256x256.png" "$HICOLOR/256x256/apps/moonlight-$BRANCH.png"
$PRIV_ESC install -Dm644 "$TEMP_DIR/icons/icon-512x512.png" "$HICOLOR/512x512/apps/moonlight-$BRANCH.png"
$PRIV_ESC install -Dm644 "$TEMP_DIR/icons/icon-1024x1024.png" "$HICOLOR/1024x1024/apps/moonlight-$BRANCH.png"

$PRIV_ESC gtk-update-icon-cache /usr/share/icons/hicolor/ || true

DESKTOP_ENTRY="[Desktop Entry]
Name=$DESKTOP_ENTRY_NAME
Comment=$DESKTOP_ENTRY_NAME Launcher
GenericName=Internet Messenger
Exec=/usr/bin/moonlight-$BRANCH
Icon=moonlight-$BRANCH
Terminal=false
Type=Application
Categories=Network;InstantMessaging;
StartupWMClass=moonlight-$BRANCH"

echo "$DESKTOP_ENTRY" | $PRIV_ESC tee "/usr/share/applications/$DESKTOP_ENTRY_FILENAME" > /dev/null

echo "Desktop entry written to /usr/share/applications/$DESKTOP_ENTRY_FILENAME"

rm -r "$TEMP_DIR"
