import { snackbar } from 'mdui/functions/snackbar'

export async function withErrorHandling<T>(
  operation: () => Promise<T>,
  successMsg?: string,
  errorMsg?: string,
): Promise<T | null> {
  try {
    const result = await operation()
    if (successMsg) snackbar({ message: successMsg })
    return result
  } catch (e) {
    snackbar({
      message: errorMsg ?? (e instanceof Error ? e.message : '操作失败'),
    })
    return null
  }
}
