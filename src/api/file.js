import { invoke } from '@tauri-apps/api/core';

export async function calculateDirSizes(paths) {
  return invoke('calculate_dir_sizes', { paths });
}

export async function calculateDirSize(path) {
  return invoke('calculate_dir_size', { path });
}

export async function startFolderSizeTask(paths) {
  return invoke('start_folder_size_task', { paths });
}

export async function getFileIcon(path) {
  return invoke('get_file_icon', { path });
}
