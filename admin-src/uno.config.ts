// uno.config.ts
import { defineConfig, presetUno, presetAttributify, presetIcons } from 'unocss'
import transformerDirectives from '@unocss/transformer-directives'

export default defineConfig({
  presets: [
    presetUno(),
    presetAttributify(),
    presetIcons({
      scale: 1.2,
      warn: true,
    }),
  ],
  shortcuts: {
    // 布局类
    'flex-center': 'flex items-center justify-center',
    'flex-between': 'flex items-center justify-between',
    'flex-col-center': 'flex flex-col items-center justify-center',
    
    // 玻璃拟态卡片
    'card-glass': 'bg-white/70 backdrop-blur-xl border border-white/40 shadow-sm rounded-2xl',
    'card-base': 'bg-white shadow-sm rounded-2xl border border-gray-100',
    'card-hover': 'transition-all duration-300 hover:shadow-lg hover:-translate-y-1',
    
    // 按钮风格
    'btn-base': 'px-4 py-2 rounded-xl transition-all duration-200 font-medium active:scale-95',
    'btn-primary': 'btn-base bg-indigo-500 text-white hover:bg-indigo-600 shadow-lg shadow-indigo-500/30',
    'btn-secondary': 'btn-base bg-white text-gray-700 border border-gray-200 hover:bg-gray-50',
    'btn-ghost': 'btn-base text-gray-600 hover:bg-gray-100/50',
    
    // 文本风格
    'text-display': 'font-bold text-gray-900 tracking-tight',
    'text-body': 'text-gray-600 leading-relaxed',
    'text-muted': 'text-gray-400 text-sm',
    
    // 交互微效
    'clickable': 'cursor-pointer select-none active:scale-95 transition-transform',
  },
  theme: {
    colors: {
      primary: {
        50: '#e0e7ff',
        100: '#c7d2fe',
        200: '#a5b4fc',
        300: '#818cf8',
        400: '#6366f1',
        500: '#4f46e5',
        600: '#4338ca',
        700: '#3730a3',
        800: '#312e81',
        900: '#1e1b4b',
      }
    },
    borderRadius: {
      'xl': '12px',
      '2xl': '20px',
      '3xl': '24px',
    }
  },
  transformers: [
    transformerDirectives(),
  ],
})
