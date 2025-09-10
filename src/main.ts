import { createApp } from "vue";
import { createPinia } from "pinia";
import ElementPlus from 'element-plus';
import 'element-plus/dist/index.css';
import * as ElementPlusIconsVue from '@element-plus/icons-vue';
import App from "./App.vue";

const app = createApp(App);
const pinia = createPinia();

// 注册Element Plus
app.use(ElementPlus);

// 注册所有图标
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component);
}

app.use(pinia);

// 添加全局错误处理
app.config.errorHandler = (err, instance, info) => {
  console.error('Vue Error:', err);
  console.error('Component instance:', instance);
  console.error('Error info:', info);
};

// 添加全局警告处理
app.config.warnHandler = (msg, instance, trace) => {
  console.warn('Vue Warning:', msg);
  console.warn('Component instance:', instance);
  console.warn('Trace:', trace);
};

// 挂载应用
try {
  app.mount("#app");
  console.log('Vue app mounted successfully');
} catch (error) {
  console.error('Failed to mount Vue app:', error);
}
