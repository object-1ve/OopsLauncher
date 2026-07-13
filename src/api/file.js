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

export async function openFileLocation(path) {
  return invoke('open_file_location', { path });
}

export async function addFavorite(path, name = '') {
  return invoke('add_favorite', { path, name });
}

export async function removeFavorite(path) {
  return invoke('remove_favorite', { path });
}

export async function getFavorites() {
  return invoke('get_favorites');
}

export async function isFavorite(path) {
  return invoke('is_favorite', { path });
}

export async function incrementOpenCount(path) {
  return invoke('increment_open_count', { path });
}
