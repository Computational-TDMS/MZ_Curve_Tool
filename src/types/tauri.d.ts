// Tauri API 类型声明
declare global {
  interface Window {
    __TAURI__: {
      invoke: (command: string, args?: any) => Promise<any>
      event: {
        listen: (event: string, handler: (event: any) => void) => Promise<void>
      }
    }
    __TAURI_INTERNALS__: any
  }
}

// Tauri 2.x Core API 类型声明
declare module '@tauri-apps/api/core' {
  export function invoke<T = any>(cmd: string, args?: Record<string, unknown>): Promise<T>
  export function transformCallback<T = any>(callback?: (response: T) => void, once?: boolean): string
}

// Tauri 2.x Event API 类型声明
declare module '@tauri-apps/api/event' {
  export interface Event<T = any> {
    event: string
    id: number
    payload: T
    windowLabel: string
  }
  
  export function listen<T = any>(
    event: string,
    handler: (event: Event<T>) => void
  ): Promise<() => void>
  
  export function emit(event: string, payload?: any): Promise<void>
}

// Tauri 2.x Plugin API 类型声明
declare module '@tauri-apps/plugin-dialog' {
  export interface OpenDialogOptions {
    title?: string
    defaultPath?: string
    filters?: Array<{
      name: string
      extensions: string[]
    }>
    multiple?: boolean
  }

  export interface SaveDialogOptions {
    title?: string
    defaultPath?: string
    filters?: Array<{
      name: string
      extensions: string[]
    }>
  }

  export function open(options?: OpenDialogOptions): Promise<string | string[] | null>
  export function save(options?: SaveDialogOptions): Promise<string | null>
  export function message(message: string, options?: {
    title?: string
    type?: 'info' | 'warning' | 'error'
  }): Promise<void>
  export function ask(message: string, options?: {
    title?: string
    type?: 'info' | 'warning' | 'error'
  }): Promise<boolean>
  export function confirm(message: string, options?: {
    title?: string
    type?: 'info' | 'warning' | 'error'
  }): Promise<boolean>
}

export {}
