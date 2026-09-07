import { musicLibrary } from '$lib/state/library.svelte'
import { settings } from '$lib/state/settings.svelte'
import { open } from '@tauri-apps/plugin-dialog'

export async function scanDirectory() {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            title: '选择音乐媒体库目录',
        })

        if (!selected || typeof selected !== 'string') return

        settings.addLibraryDir(selected)

        await musicLibrary.scan()
    } catch (err) {
        console.error('扫描音乐媒体库目录失败:', err)
    }
}

export async function removeMusicDirectory(dir: string) {
    settings.removeLibraryDir(dir)
    await musicLibrary.scan()
}