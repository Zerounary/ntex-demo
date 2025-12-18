import { createApp } from 'vue';
import App from './App.vue';
import '@purge-icons/generated';
import { createI18n } from 'vue-i18n';

import './styles/design-tokens.css';
import './styles/animations.css';
import './styles/base.css';

// Router
import { Router } from '@/router';

// i18n
import messages from '@intlify/vite-plugin-vue-i18n/messages';


// Pinia
import {createPinia} from "pinia";
import piniaPersist from 'pinia-plugin-persist'

const i18n = createI18n({
  locale: 'en',
  messages,
});

// Unocss
import 'virtual:uno.css'

// WindiCSS
// import 'virtual:windi.css';
// import 'virtual:windi-devtools';


// plugins
import Vue3TouchEvents from 'vue3-touch-events';

const app = createApp(App);

app.use(i18n);

app.use(Router);

app.use(Vue3TouchEvents)

const pinia = createPinia()
pinia.use(piniaPersist)
app.use(pinia)

app.mount('#app');
