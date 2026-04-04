import { browser } from '$app/environment'

export class Router {
  path = $state(browser ? window.location.pathname : '/')

  constructor() {
    if (browser) {
      window.addEventListener('popstate', () => {
        this.path = window.location.pathname
      })
    }
  }

  navigate(path: string) {
    if (browser) {
      window.history.pushState({}, '', path)
      this.path = path
      window.dispatchEvent(new Event('pathchange'))
    }
  }
}

export const router = new Router()
