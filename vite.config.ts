import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => {
  const plugins = [vue()];
  
  // Vue DevTools support (only in development)
  if (process.env.NODE_ENV === 'development' || process.env.DEV) {
    try {
      const { default: VueDevtools } = await import('@vue/devtools');
      plugins.push(VueDevtools());
      console.log('Vue DevTools enabled');
    } catch (error) {
      console.warn('Vue DevTools not available:', error);
    }
  }
  
  return {
    plugins,

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent Vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
      port: 1420,
      strictPort: true,
      host: host || false,
      hmr: host
        ? {
            protocol: "ws",
            host,
            port: 1421,
          }
        : undefined,
      watch: {
        // 3. tell Vite to ignore watching `src-tauri`
        ignored: ["**/src-tauri/**"],
      },
    },

    // 构建优化配置
    build: {
      // 调整 chunk 大小警告限制
      chunkSizeWarningLimit: 1000,
      
      // 手动配置代码分割
      rollupOptions: {
        output: {
          manualChunks: (id) => {
            // 将 Vue 相关库分离
            if (id.includes('vue') || id.includes('@vue')) {
              return 'vue-vendor';
            }
            
            // 将 Element Plus 分离
            if (id.includes('element-plus') || id.includes('@element-plus')) {
              return 'element-plus';
            }
            
            // 将 Plotly.js 分离（这是最大的依赖）
            if (id.includes('plotly')) {
              return 'plotly';
            }
            
            // 将 Tauri API 分离
            if (id.includes('@tauri-apps')) {
              return 'tauri';
            }
            
            // 将 node_modules 中的其他库分离
            if (id.includes('node_modules')) {
              return 'vendor';
            }
          },
          
          // 优化 chunk 文件名
          chunkFileNames: 'js/[name]-[hash].js',
          
          // 优化资源文件名
          assetFileNames: (assetInfo) => {
            const info = assetInfo.name.split('.');
            const ext = info[info.length - 1];
            if (/\.(css)$/.test(assetInfo.name)) {
              return `css/[name]-[hash].${ext}`;
            }
            return `assets/[name]-[hash].${ext}`;
          }
        }
      },
      
      // 启用 CSS 代码分割
      cssCodeSplit: true,
      
      // 使用 esbuild 进行压缩（更快）
      minify: 'esbuild',
      
      // 或者使用 terser（更小的文件大小）
      // minify: 'terser',
      // terserOptions: {
      //   compress: {
      //     // 移除 console.log（生产环境）
      //     drop_console: true,
      //     drop_debugger: true,
      //   },
      // },
    },
  };
});