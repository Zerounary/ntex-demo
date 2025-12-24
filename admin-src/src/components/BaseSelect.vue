<template>
  <div class="base-select relative" ref="containerRef">
    <!-- Label -->
    <label v-if="label" class="block text-xs font-medium text-gray-600 mb-1.5">
      {{ label }} <span v-if="required" class="text-red-400">*</span>
    </label>

    <!-- Trigger -->
    <div
      ref="triggerRef"
      @click="toggle"
      class="w-full px-3 py-2.5 bg-white border rounded-xl text-sm transition-all duration-200 cursor-pointer flex items-center justify-between group"
      :class="[
        isOpen 
          ? 'border-primary-400 ring-2 ring-primary-400/20 shadow-sm' 
          : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50/50',
        disabled ? 'opacity-60 cursor-not-allowed bg-gray-50' : ''
      ]"
    >
      <div class="flex items-center gap-2 overflow-hidden">
        <!-- Icon slot -->
        <div v-if="$slots.icon" class="text-gray-400 group-hover:text-primary-500 transition-colors">
          <slot name="icon"></slot>
        </div>
        
        <!-- Selected Value -->
        <span v-if="selectedOption" class="truncate text-gray-700 font-medium">
          {{ selectedOption.label }}
        </span>
        <span v-else class="text-gray-400 truncate">
          {{ placeholder }}
        </span>
      </div>

      <!-- Arrow -->
      <div 
        class="text-gray-400 transition-transform duration-300"
        :class="isOpen ? 'rotate-180 text-primary-500' : ''"
      >
        <div class="i-carbon-chevron-down text-lg"></div>
      </div>
    </div>

    <!-- Dropdown Menu -->
    <teleport to="body">
      <transition
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="transform scale-95 opacity-0 -translate-y-2"
        enter-to-class="transform scale-100 opacity-100 translate-y-0"
        leave-active-class="transition duration-150 ease-in"
        leave-from-class="transform scale-100 opacity-100 translate-y-0"
        leave-to-class="transform scale-95 opacity-0 -translate-y-2"
      >
        <div
          v-if="isOpen"
          ref="dropdownRef"
          class="fixed z-[9999] bg-white rounded-xl shadow-xl border border-gray-100 py-1 overflow-hidden focus:outline-none"
          :style="dropdownStyle"
        >
          <!-- Search (Optional) -->
          <div v-if="searchable" class="px-2 pb-1 pt-2">
             <div class="relative">
               <input
                 v-model="searchQuery"
                 ref="searchInputRef"
                 type="text"
                 class="w-full pl-8 pr-3 py-1.5 text-sm bg-gray-50 border border-gray-200 rounded-lg focus:outline-none focus:border-primary-400 focus:bg-white transition-colors"
                 placeholder="Search..."
                 @click.stop
               />
               <div class="absolute left-2.5 top-1/2 -translate-y-1/2 text-gray-400">
                 <div class="i-carbon-search"></div>
               </div>
             </div>
          </div>

          <!-- Options List -->
          <ul class="max-h-60 overflow-auto custom-scrollbar p-1">
            <li
              v-for="option in filteredOptions"
              :key="option.value"
              @click="select(option)"
              class="relative px-3 py-2 rounded-lg cursor-pointer text-sm transition-colors flex items-center justify-between group"
              :class="[
                modelValue === option.value
                  ? 'bg-primary-50 text-primary-700 font-medium'
                  : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
              ]"
            >
              <div class="flex items-center gap-2">
                 <div v-if="option.icon" :class="option.icon" class="text-lg opacity-70 group-hover:opacity-100"></div>
                 <span>{{ option.label }}</span>
              </div>
              
              <div v-if="modelValue === option.value" class="text-primary-600 animate-scale-in">
                <div class="i-carbon-checkmark"></div>
              </div>
            </li>
            
            <li v-if="filteredOptions.length === 0" class="px-3 py-4 text-center text-sm text-gray-400">
              No options found
            </li>
          </ul>
        </div>
      </transition>
    </teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue';

interface Option {
  label: string;
  value: any;
  icon?: string;
  [key: string]: any;
}

interface Props {
  modelValue: any;
  options: Option[];
  label?: string;
  placeholder?: string;
  required?: boolean;
  disabled?: boolean;
  searchable?: boolean;
}

interface Emits {
  (e: 'update:modelValue', value: any): void;
  (e: 'change', value: any): void;
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: 'Select an option',
  required: false,
  disabled: false,
  searchable: false,
});

const emit = defineEmits<Emits>();

const isOpen = ref(false);
const containerRef = ref<HTMLElement | null>(null);
const triggerRef = ref<HTMLElement | null>(null);
const dropdownRef = ref<HTMLElement | null>(null);
const searchInputRef = ref<HTMLInputElement | null>(null);
const searchQuery = ref('');

const dropdownStyle = ref<Record<string, string>>({});

const selectedOption = computed(() => {
  return props.options.find(opt => opt.value === props.modelValue);
});

const filteredOptions = computed(() => {
  if (!props.searchable || !searchQuery.value) return props.options;
  const query = searchQuery.value.toLowerCase();
  return props.options.filter(opt => 
    opt.label.toLowerCase().includes(query)
  );
});

const toggle = () => {
  if (props.disabled) return;
  isOpen.value = !isOpen.value;
  if (isOpen.value) nextTick(() => updateDropdownPosition());
  if (isOpen.value && props.searchable) {
    searchQuery.value = '';
    nextTick(() => {
      searchInputRef.value?.focus();
    });
  }
};

const select = (option: Option) => {
  emit('update:modelValue', option.value);
  emit('change', option.value);
  isOpen.value = false;
};

const updateDropdownPosition = () => {
  const el = triggerRef.value || containerRef.value;
  if (!el) return;
  const rect = el.getBoundingClientRect();
  const dropdownEl = dropdownRef.value;
  const margin = 8;
  const viewportHeight = window.innerHeight;
  const viewportWidth = window.innerWidth;
  const dropdownHeight = dropdownEl?.offsetHeight || 0;
  const dropdownWidth = rect.width;
  let top = rect.bottom + margin;
  if (dropdownHeight && top + dropdownHeight > viewportHeight - margin) {
    const flippedTop = rect.top - dropdownHeight - margin;
    if (flippedTop >= margin) top = flippedTop;
    else top = Math.max(margin, viewportHeight - dropdownHeight - margin);
  }
  let left = rect.left;
  const maxLeft = viewportWidth - dropdownWidth - margin;
  if (left > maxLeft) left = Math.max(margin, maxLeft);
  dropdownStyle.value = {
    left: `${left}px`,
    top: `${top}px`,
    width: `${dropdownWidth}px`,
  };
};

const handleClickOutside = (event: MouseEvent) => {
  const t = event.target as Node;
  const inTrigger = containerRef.value ? containerRef.value.contains(t) : false;
  const inDropdown = dropdownRef.value ? dropdownRef.value.contains(t) : false;
  if (!inTrigger && !inDropdown) isOpen.value = false;
};

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
  window.addEventListener('resize', updateDropdownPosition);
  window.addEventListener('scroll', updateDropdownPosition, true);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
  window.removeEventListener('resize', updateDropdownPosition);
  window.removeEventListener('scroll', updateDropdownPosition, true);
});

watch(
  () => isOpen.value,
  (open) => {
    if (open) nextTick(() => updateDropdownPosition());
  }
);
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: #e2e8f0;
  border-radius: 20px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background-color: #cbd5e1;
}
</style>
