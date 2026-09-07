<script lang="ts" generics="T extends string">
    import SearchField from '$lib/components/base/SearchField.svelte'
    import Select from '$lib/components/base/Select.svelte'

    type SortOption<T extends string> = {
        label: string
        value: T
    }

    type LibraryFiltersProps<T extends string> = {
        query?: string
        sortBy?: T
        searchPlaceholder?: string
        sortOptions: SortOption<T>[]
        class?: string
    }

    let {
        query = $bindable(''),
        sortBy = $bindable<T>(),
        searchPlaceholder = '搜索...',
        sortOptions,
        class: className = '',
    }: LibraryFiltersProps<T> = $props()
</script>

<div class={`flex gap-3 md:flex-row md:items-center p-2 ${className}`}>
    <SearchField
        bind:value={query}
        placeholder={searchPlaceholder}
    />

    <Select
        bind:value={sortBy}
        options={sortOptions}
    />
</div>