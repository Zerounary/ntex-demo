// uno.config.ts
import { defineConfig } from 'unocss'
import transformerDirectives from '@unocss/transformer-directives'

export default defineConfig({
    // ...UnoCSS options
    rules: [
        ['layout', { padding: '30px', 'background-color': 'blue', color: 'white' }],
    ],
    transformers: [
        transformerDirectives(),
    ],
})