export interface ExtensionNavItem {
    id: string
    title: string
    icon: string
    href: string
    order?: number
}

export interface ExtensionCommand {
    id: string
    title: string
    run: () => void | Promise<void>
}

export interface AppExtension {
    id: string
    name: string
    description?: string
    enabled?: boolean
    navItems?: ExtensionNavItem[]
    commands?: ExtensionCommand[]
}