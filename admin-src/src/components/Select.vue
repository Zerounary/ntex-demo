<template>
  <div class="text-blue font-bold" @click="show">
    {{ text }}
  </div>
  <div
    v-if="isShow"
    class="w-screen h-400px absolute box-border bottom-0 bg-white rounded-lg overflow-hidden border border-solid border-gray-400 shadow"
  >
    <div class="flex flex-col p-3 space-y-3 h-300px overflow-y-auto">
      <template v-for="g in group" :key="g.group">
        <div
          class="p-3 border border-solid border-gray rounded"
          :class="{
            'bg-green-300 text-green-900 border-green-300': isActive(g),
          }"
          @click="select(g)"
        >
          <div class="text-xl font-bold">{{ g.group }}</div>
          <div class="mt-3 text-sm">{{ g.desc }}</div>
        </div>
      </template>
    </div>
    <div class="h-30px text-xs mx-3 flex items-center justify-center">
      当前选择：{{ target?.group || '请选择' }}
    </div>
    <div class="flex justify-between items-center">
      <div class="btn text-gray-500" @click="cancel">取消</div>
      <div class="btn text-green-500" @click="confirm">确定</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import group from '/public/group.json';
import { computed, defineEmits } from 'vue';

const text = ref('请选择');
const target = ref(null);
const isShow = ref(false);
const emit = defineEmits(['select']);

const show = () => {
  isShow.value = true;
};

const select = g => {
  target.value = g;
};

const isActive = computed(() => {
  return e => target.value?.group === e.group;
});

const confirm = () => {
  if (!target.value?.group) {
    alert('请先选择轮相组');
    return;
  }
  text.value = target.value?.group;
  isShow.value = false;
  emit('select', target.value);
};
const cancel = () => {
  isShow.value = false;
  target.value = null;
};
</script>

<style scoped>
.btn {
  @apply w-70px text-center;
}
</style>
