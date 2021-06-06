set -e

export APPIMAGE_EXTRACT_AND_RUN=1
wget https://github.com/TheAssassin/appimagecraft/releases/download/continuous/appimagecraft-x86_64.AppImage
chmod +x appimagecraft-x86_64.AppImage
./appimagecraft-x86_64.AppImage
