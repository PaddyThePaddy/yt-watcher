<script setup lang="ts">
const prop = defineProps({
  state: {
    type: String,
    required: true
  },
  profile_img: String,
  handle: String,
  display_name: String,
  error_msg: String
})
</script>

<template>
  <div v-if="prop.state != 'none'">
    <div v-if="prop.state == 'loading'">
      <div class="lds-ring">
        <div></div>
        <div></div>
        <div></div>
        <div></div>
      </div>
    </div>
    <div v-if="prop.state == 'loaded'">
      <img :src="prop.profile_img" class="channel_icon" />
      <br />
      <span class="channel_handle">{{ prop.handle }}</span>
      <br />
      <span class="chanel_display_name">{{ prop.display_name }}</span
      ><br />
      <button @click="$emit('follow')">Follow</button>
    </div>
    <div v-if="prop.state == 'error'">
      <span class="error_msg">{{ prop.error_msg }}</span>
    </div>
  </div>
</template>

<style scoped>
.lds-ring {
  display: inline-block;
  position: relative;
  width: 80px;
  height: 80px;
}
.lds-ring div {
  box-sizing: border-box;
  display: block;
  position: absolute;
  width: 64px;
  height: 64px;
  margin: 8px;
  border: 8px solid #fff;
  border-radius: 50%;
  animation: lds-ring 1.2s cubic-bezier(0.5, 0, 0.5, 1) infinite;
  border-color: #fff transparent transparent transparent;
}
.lds-ring div:nth-child(1) {
  animation-delay: -0.45s;
}
.lds-ring div:nth-child(2) {
  animation-delay: -0.3s;
}
.lds-ring div:nth-child(3) {
  animation-delay: -0.15s;
}
@keyframes lds-ring {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

img.channel_icon {
  border-radius: 50%;
  max-width: 240px;
}

.error_msg {
  font-weight: bold;
  color: ghostwhite;
}
</style>
