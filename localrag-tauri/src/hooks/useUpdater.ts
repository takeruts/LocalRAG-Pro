import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { useState, useCallback } from 'react';

export function useUpdater() {
  const [isChecking, setIsChecking] = useState(false);
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [updateVersion, setUpdateVersion] = useState<string | null>(null);
  const [isDownloading, setIsDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState(0);

  const checkForUpdates = useCallback(async () => {
    setIsChecking(true);
    try {
      const update = await check();
      if (update) {
        setUpdateAvailable(true);
        setUpdateVersion(update.version);
        return update;
      }
      setUpdateAvailable(false);
      setUpdateVersion(null);
      return null;
    } catch (error) {
      console.error('Failed to check for updates:', error);
      return null;
    } finally {
      setIsChecking(false);
    }
  }, []);

  const downloadAndInstall = useCallback(async () => {
    setIsDownloading(true);
    setDownloadProgress(0);

    try {
      const update = await check();
      if (update) {
        await update.downloadAndInstall((event) => {
          switch (event.event) {
            case 'Started':
              setDownloadProgress(0);
              break;
            case 'Progress':
              // Progress event has chunkLength
              setDownloadProgress((prev) => Math.min(prev + 10, 90));
              break;
            case 'Finished':
              setDownloadProgress(100);
              break;
          }
        });
        await relaunch();
      }
    } catch (error) {
      console.error('Failed to download and install update:', error);
    } finally {
      setIsDownloading(false);
    }
  }, []);

  return {
    isChecking,
    updateAvailable,
    updateVersion,
    isDownloading,
    downloadProgress,
    checkForUpdates,
    downloadAndInstall,
  };
}
