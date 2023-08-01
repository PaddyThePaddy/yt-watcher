<script setup lang="ts">
import { computed, ref, type StyleValue, type ComputedRef, type Ref } from 'vue'
import * as utils from '../utils'
const emit = defineEmits(['show_popup', 'update_video_list', 'set_sync_key'])

const prop = defineProps({
  yt_thumbnail: String,
  yt_handle: String,
  yt_display_name: String,

  tw_thumbnail: String,
  tw_handle: String,
  tw_display_name: String,

  sync_key: {
    type: String,
    required: true
  },
  sub_yt_channels: {
    type: Array<string>,
    required: true
  },
  sub_tw_channels: {
    type: Array<string>,
    required: true
  }
})
const alarm_enabled = ref(false)
const sync_key: Ref<string> = ref(prop.sync_key)
const is_sync_key_valid = computed(() => {
  return utils.verify_sync_key(sync_key.value)
})

function copy_calendar_url() {
  if (prop.sub_yt_channels.length == 0 && prop.sub_tw_channels.length == 0) {
    console.log('No id for calendar to copy')
    return
  }
  let url = utils.site_url + 'cal?'
  if (prop.sub_yt_channels.length != 0) {
    url += 'yt-ch=' + prop.sub_yt_channels.join(',')
  }
  if (prop.sub_tw_channels.length != 0) {
    if (prop.sub_yt_channels.length != 0) {
      url += '&'
    }
    url += 'tw-ch=' + prop.sub_tw_channels.join(',')
  }
  if (alarm_enabled.value) {
    url += '&alarm=true'
  }
  navigator.clipboard.writeText(url).then(() => {
    console.log('copy success')
    emit('show_popup', 'Copied')
  })
}

function copy_synced_calendar_url() {
  if (prop.sync_key == null || prop.sync_key.length == 0 || !utils.verify_sync_key(prop.sync_key)) {
    console.log('invalid sync key')
    return
  }
  let url = utils.site_url + 'cal?key=' + prop.sync_key
  if (alarm_enabled.value) {
    url += '&alarm=true'
  }
  navigator.clipboard.writeText(url).then(() => {
    console.log('copy success')
    emit('show_popup', 'Copied')
  })
}

function new_sync_key() {
  utils.new_sync_key().then((key) => {
    console.log(key)
    if (key != null) {
      sync_key.value = key
      emit('set_sync_key', key)
    }
  })
}

function push_sync_key() {
  if (sync_key.value != null && utils.verify_sync_key(sync_key.value)) {
    utils.push_sync_key(sync_key.value, prop.sub_yt_channels, prop.sub_tw_channels)
  }
}

function on_sync_key_changed() {
  if (utils.verify_sync_key(sync_key.value)) {
    emit('set_sync_key', sync_key.value)
  }
}
</script>
<template>
  <div id="menu_header"></div>
  <div>
    <b style="margin: 5px; display: block">Sync Key</b>
    <input
      type="text"
      v-model="sync_key"
      placeholder="Synchronize Key"
      :class="{ error: sync_key.length > 0 && !utils.verify_sync_key(sync_key) }"
      @keyup="on_sync_key_changed"
    /><br />
    <button @click="new_sync_key">New</button>
    <button v-if="is_sync_key_valid" @click="$emit('pull_sync_key')">Pull</button>
    <button v-if="is_sync_key_valid" @click="push_sync_key">Push</button>
  </div>
  <br />
  <div>
    <button @click="copy_calendar_url">Copy calendar url</button>
    <button @click="copy_synced_calendar_url">Copy synced calendar url</button>
    <div style="margin-top: 5px; margin-bottom: 5px">
      <input id="alarm" type="checkbox" v-model="alarm_enabled" hidden="true" />
      <label for="alarm" :class="{ checked: alarm_enabled }">Alarm</label>
    </div>
  </div>
  <hr />
  <span>Tracking {{ prop.sub_yt_channels.length + prop.sub_tw_channels.length }} channels</span>
  <ul>
    <li v-for="(ch, idx) in prop.sub_yt_channels" v-bind:key="idx" class="channel_id">
      <button class="del_btn" @click="$emit('unfollow_yt_ch', ch)">x</button>
      {{ ch }}&nbsp;<a :href="'https://www.youtube.com/' + ch">
        <img class="platform_icon" src="/youtube_32x32.png"
      /></a>
    </li>
    <li v-for="(ch, idx) in prop.sub_tw_channels" v-bind:key="idx" class="channel_id">
      <button class="del_btn" @click="$emit('unfollow_tw_ch', ch)">x</button>
      {{ ch }}&nbsp;<a :href="'https://www.twitch.tv/' + ch">
        <img class="platform_icon" src="/twitch_32x32.png"
      /></a>
    </li>
  </ul>
  <button @click="$emit('clear_all_channel')">Clear All</button>
  <hr />
  <p>
    About:<br />
    這是我為了自己看 vtuber 需求做的小工具<br />
    在這個網頁可以查詢、追蹤想要的頻道，並且提供 iCal 連結讓其他行事曆 App 匯入<br />
    本意是模仿 Holodex，但是可以用來追蹤訂閱數通常低於 Holodex 註冊門檻的小頻道<br />
    因為這個網站是架在一個超爛雲端主機上，效能非常有限，如果想要追蹤訂閱數高於兩萬的 vtuber
    頻道，還是推薦用功能比較完整的
    <a href="https://holodex.net">Holodex</a><br />
    <br />
    This is a basic tool made for myself.<br />
    It can track youtube channels ongoing/upcoming streams and provide a iCal resource to add to
    calendar apps.<br />
    This project is highly inspired by Holodex, but can be used to track small channels that doesn't
    meet the register threshold of Holodex.<br />
    This site is ran on the most basic cloud machine I can find, so if you want to track channels
    that has over 20k subs,
    <a href="https://holodex.net">Holodex</a> is still recommended.<br />
    <br />
    Contact: paddycup1 on discord <br />
    Source:
    <a href="https://github.com/PaddyThePaddy/yt-watcher"
      >https://github.com/PaddyThePaddy/yt-watcher</a
    ><br /><br />
    <span style="font-size: 0.7em"
      >Refresh and menu icon by <a href="https://icons8.com">Icons8</a></span
    >
  </p>
  <div id="menu_footer"></div>
</template>

<style scoped>
img.channel_icon {
  border-radius: 50%;
  max-width: 240px;
}

img.platform_icon {
  height: 1.2em;
  vertical-align: middle;
}

li.channel_id {
  list-style-type: none;
}

button.del_btn {
  padding-top: 0px;
  padding-bottom: 0px;
  padding-left: 5px;
  padding-right: 5px;
}

input {
  padding-left: 5px;
  padding-right: 5px;
  border: solid rgb(0, 101, 135) 2px;
}

input.error {
  outline: red solid 3px;
}

label {
  margin: 5px;
  border: solid whitesmoke 1px;
  padding-left: 5px;
  padding-right: 5px;
  user-select: none;
}

label:hover {
  border: solid rgb(104, 217, 255) 1px;
  color: #ffffff;
}
label.checked {
  background-color: rgb(20, 139, 179);
  border: solid rgb(20, 139, 179) 2px;
}

@media (pointer: fine) {
  div#menu_footer {
    height: 0px;
    visibility: hidden;
  }
  div#menu_header {
    height: 60px;
    visibility: visible;
  }
}
@media (pointer: none), (pointer: coarse) {
  div#menu_footer {
    height: 60px;
    visibility: visible;
  }
  div#menu_header {
    height: 0px;
    visibility: hidden;
  }
}
</style>
