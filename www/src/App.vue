<script setup lang="ts">
import VideoComponent from './components/VideoComponent.vue'
import SideBar from './components/SideBar.vue'
import ChannelProfile from './components/ChannelProfile.vue'
import * as utils from './utils'
import { computed, ref, type ComputedRef, type Ref, type StyleValue } from 'vue'

type VideoEvent = {
  target_url: string
  thumbnail_url: string
  title: string
  start_time: Date
  current_time: Date | null
  ongoing: boolean
  source_name: string
  source_profile_img_url: string
  source_url: string
  source_type: string
}

let side_bar_props = ref({
  yt_thumbnail: '',
  yt_handle: '',
  yt_display_name: '',
  tw_thumbnail: '',
  tw_handle: '',
  tw_display_name: '',
  sync_key: utils.get_sync_key(),
  sub_yt_channels: utils.get_yt_id_list(),
  sub_tw_channels: utils.get_tw_id_list()
})
let upcoming_videos: Ref<VideoEvent[]> = ref([])
let filtered_upcoming_videos: ComputedRef<VideoEvent[]> = computed(() => {
  return upcoming_videos.value.filter(
    (v) =>
      search_bar_val.value.trim().length == 0 ||
      v.title.indexOf(search_bar_val.value) != -1 ||
      v.source_name.indexOf(search_bar_val.value) != -1 ||
      v.source_url.indexOf(search_bar_val.value) != -1 ||
      v.target_url.indexOf(search_bar_val.value) != -1
  )
})
let starting_videos: Ref<VideoEvent[]> = ref([])
let filtered_starting_videos: ComputedRef<VideoEvent[]> = computed(() => {
  return starting_videos.value.filter(
    (v) =>
      search_bar_val.value.trim().length == 0 ||
      v.title.indexOf(search_bar_val.value) != -1 ||
      v.source_name.indexOf(search_bar_val.value) != -1 ||
      v.source_url.indexOf(search_bar_val.value) != -1 ||
      v.target_url.indexOf(search_bar_val.value) != -1
  )
})
let ongoing_videos: Ref<VideoEvent[]> = ref([])
let filtered_ongoing_videos: ComputedRef<VideoEvent[]> = computed(() => {
  return ongoing_videos.value.filter(
    (v) =>
      search_bar_val.value.trim().length == 0 ||
      v.title.indexOf(search_bar_val.value) != -1 ||
      v.source_name.indexOf(search_bar_val.value) != -1 ||
      v.source_url.indexOf(search_bar_val.value) != -1 ||
      v.target_url.indexOf(search_bar_val.value) != -1
  )
})
let current_time: Ref<Date> = ref(new Date())
const sidebar_control = ref(false)
const sidebar_reactive_style: ComputedRef<StyleValue> = computed(() => {
  if (sidebar_control.value) {
    return { width: 'var(--menu-width)', padding: '5px', overflowY: 'scroll' }
  } else {
    return { width: '0px', padding: '0px', overflowY: 'hidden' }
  }
})
const search_bar_val = ref('')
const yt_channel_state = ref('none')
const yt_channel_data: Ref<{
  display_name: string
  handle: string
  profile_img: string
  error_msg: string
}> = ref({
  display_name: '',
  handle: '',
  profile_img: '',
  error_msg: ''
})
const tw_channel_state = ref('none')
const tw_channel_data: Ref<{
  display_name: string
  handle: string
  profile_img: string
  error_msg: string
}> = ref({
  display_name: '',
  handle: '',
  profile_img: '',
  error_msg: ''
})
const popup_msgs: Ref<{ msg: string; time: number }[]> = ref([])
setInterval(
  () => {
    update_video_events()
  },
  1000 * 60 * 5
)
setInterval(() => {
  current_time.value = new Date()
}, 50)

setInterval(() => {
  for (let i = popup_msgs.value.length - 1; i >= 0; i--) {
    if (current_time.value.getTime() - popup_msgs.value[i].time > 5000) {
      popup_msgs.value.splice(i, 1)
    }
  }
}, 1000)

side_bar_props.value.sub_tw_channels.sort()
side_bar_props.value.sub_yt_channels.sort()

let touch_start_pos: null | { x: number; y: number } = null

function touch_start(event: TouchEvent) {
  touch_start_pos = { x: event.touches[0].clientX, y: event.touches[0].clientY }
}

function touch_move(event: TouchEvent) {
  if (touch_start_pos == null) {
    return
  }

  const current = { x: event.touches[0].clientX, y: event.touches[0].clientY }
  const diff_x = touch_start_pos.x - current.x
  const diff_y = touch_start_pos.y - current.y

  if (Math.abs(diff_x) > Math.abs(diff_y)) {
    if (diff_x > 150) {
      sidebar_control.value = false
      touch_start_pos = null
    } else if (diff_x < -150) {
      sidebar_control.value = true
      touch_start_pos = null
    }
  } else {
    if (Math.abs(diff_y) > 150) {
      touch_start_pos = null
    }
  }
}
function load_youtube(query: string) {
  yt_channel_state.value = 'loading'
  utils.load_youtube_channel(query).then(
    (data) => {
      console.log(data)
      yt_channel_data.value.display_name = data.display
      yt_channel_data.value.handle = data.handle
      yt_channel_data.value.profile_img = data.thumbnail
      yt_channel_state.value = 'loaded'
    },
    (error) => {
      console.log(error)
      yt_channel_data.value.error_msg = error
      yt_channel_state.value = 'error'
    }
  )
}

function follow_youtube() {
  if (yt_channel_data.value.handle == null || yt_channel_data.value.handle.length == 0) {
    return
  }
  let handle = yt_channel_data.value.handle
  if (handle.startsWith('@')) {
    handle = handle.substring(1)
  }
  if (side_bar_props.value.sub_yt_channels.indexOf(handle) == -1) {
    side_bar_props.value.sub_yt_channels.push(handle)
    side_bar_props.value.sub_yt_channels.sort()
    utils.set_yt_id_list(side_bar_props.value.sub_yt_channels)
    update_video_events()
    show_popup('Followed')
  } else {
    show_popup(yt_channel_data.value.handle + ' already followed')
    console.log(yt_channel_data.value.handle + ' already followed')
  }
}

function load_twitch(query: string) {
  tw_channel_state.value = 'loading'
  utils.load_twitch_channel(query).then(
    (data) => {
      tw_channel_data.value.display_name = data.display
      tw_channel_data.value.handle = data.handle
      tw_channel_data.value.profile_img = data.thumbnail
      tw_channel_state.value = 'loaded'
    },
    (error) => {
      console.log(error)
      tw_channel_data.value.error_msg = error
      tw_channel_state.value = 'error'
    }
  )
}

function follow_twitch() {
  if (tw_channel_data.value.handle == null || tw_channel_data.value.handle.length == 0) {
    return
  }

  if (side_bar_props.value.sub_tw_channels.indexOf(tw_channel_data.value.handle) == -1) {
    side_bar_props.value.sub_tw_channels.push(tw_channel_data.value.handle)
    side_bar_props.value.sub_tw_channels.sort()
    utils.set_tw_id_list(side_bar_props.value.sub_tw_channels)
    update_video_events()
    show_popup('Followed')
  } else {
    show_popup(tw_channel_data.value.handle + ' already followed')
    console.log(tw_channel_data.value.handle + ' already followed')
  }
}

function update_video_events() {
  const new_ongoing_videos: VideoEvent[] = []
  const new_starting_videos: VideoEvent[] = []
  const new_upcoming_videos: VideoEvent[] = []
  console.log('Updating video list')
  return utils
    .get_video_data(side_bar_props.value.sub_yt_channels, side_bar_props.value.sub_tw_channels)
    .then((data: utils.UpcomingEvent[]) => {
      const now = new Date()
      for (const event of data) {
        let source = ''
        let source_name = ''
        let source_profile_img = ''
        let source_url = ''
        const start_time = new Date(event.start_timestamp_millis)
        if (event.source.TwitchChannel != null) {
          source = 'twitch'
          source_name = event.source.TwitchChannel.title
          source_profile_img = event.source.TwitchChannel.thumbnail_url
          source_url = 'https://www.twitch.tv/' + event.source.TwitchChannel.login
        } else if (event.source.YoutubeChannel != null) {
          source = 'youtube'
          source_name = event.source.YoutubeChannel.title
          source_profile_img = event.source.YoutubeChannel.thumbnail_url
          source_url = 'https://www.youtube.com/' + event.source.YoutubeChannel.custom_url
        }
        const video = {
          target_url: event.target_url,
          thumbnail_url: event.thumbnail_url,
          title: event.title,
          start_time: start_time,
          current_time: null,
          ongoing: event.ongoing,
          source_name: source_name,
          source_profile_img_url: source_profile_img,
          source_url: source_url,
          source_type: source
        }
        if (event.ongoing) {
          new_ongoing_videos.push(video)
        } else if (start_time < now) {
          new_starting_videos.push(video)
        } else {
          new_upcoming_videos.push(video)
        }
      }
      ongoing_videos.value = new_ongoing_videos
      starting_videos.value = new_starting_videos
      upcoming_videos.value = new_upcoming_videos
    })
}

function set_sync_key(key: string) {
  if (utils.verify_sync_key(key)) {
    side_bar_props.value.sync_key = key
    utils.set_sync_key(key)
  }
}

function pull_sync_key() {
  if (!utils.verify_sync_key(side_bar_props.value.sync_key)) {
    return
  }
  utils.pull_sync_key(side_bar_props.value.sync_key).then(
    (resp) => {
      for (const ch of resp.yt_ch) {
        if (side_bar_props.value.sub_yt_channels.indexOf(ch) == -1) {
          side_bar_props.value.sub_yt_channels.push(ch)
        }
      }
      side_bar_props.value.sub_yt_channels.sort()
      utils.set_yt_id_list(side_bar_props.value.sub_yt_channels)
      for (const ch of resp.tw_ch) {
        if (side_bar_props.value.sub_tw_channels.indexOf(ch) == -1) {
          side_bar_props.value.sub_tw_channels.push(ch)
        }
      }
      side_bar_props.value.sub_tw_channels.sort()
      utils.set_tw_id_list(side_bar_props.value.sub_tw_channels)
      update_video_events()
      console.log('pulled')
      show_popup('Pull succeed')
    },
    (reject) => {
      show_popup(reject)
    }
  )
}

function unfollow_yt_ch(ch: string) {
  let idx = side_bar_props.value.sub_yt_channels.indexOf(ch)
  if (idx != -1) {
    side_bar_props.value.sub_yt_channels.splice(idx, 1)
    utils.set_yt_id_list(side_bar_props.value.sub_yt_channels)
    update_video_events()
  }
}

function unfollow_tw_ch(ch: string) {
  let idx = side_bar_props.value.sub_tw_channels.indexOf(ch)
  if (idx != -1) {
    side_bar_props.value.sub_tw_channels.splice(idx, 1)
    utils.set_tw_id_list(side_bar_props.value.sub_tw_channels)
    update_video_events()
  }
}

function clear_all_ch() {
  side_bar_props.value.sub_yt_channels = []
  side_bar_props.value.sub_tw_channels = []
  utils.set_tw_id_list([])
  utils.set_yt_id_list([])
  update_video_events()
}

function clear_ch_preview() {
  side_bar_props.value.tw_display_name = ''
  side_bar_props.value.tw_handle = ''
  side_bar_props.value.tw_thumbnail = ''
  side_bar_props.value.yt_display_name = ''
  side_bar_props.value.yt_handle = ''
  side_bar_props.value.yt_thumbnail = ''
}

function import_list(list: string) {
  const yt_list = new RegExp('.+(?:\\?|&)yt-ch=([^&]+)').exec(list)
  let promises = []
  let imported_count = 0
  if (yt_list != null) {
    for (const ch of yt_list[1].split(',').filter((s) => s.length != 0)) {
      promises.push(
        utils.load_youtube_channel(ch).then((info) => {
          let handle = info.handle
          if (handle.startsWith('@')) {
            handle = handle.substring(1)
          }
          if (side_bar_props.value.sub_yt_channels.indexOf(handle) == -1) {
            side_bar_props.value.sub_yt_channels.push(handle)
            imported_count += 1
          }
        })
      )
    }
  }
  const tw_list = new RegExp('.+(?:\\?|&)tw-ch=([^&]+)').exec(list)
  if (tw_list != null) {
    for (const ch of tw_list[1].split(',').filter((s) => s.length != 0)) {
      promises.push(
        utils.load_twitch_channel(ch).then((info) => {
          if (side_bar_props.value.sub_tw_channels.indexOf(info.handle) == -1) {
            side_bar_props.value.sub_tw_channels.push(info.handle)
            imported_count += 1
          }
        })
      )
    }
  }
  Promise.all(promises).then(() => {
    side_bar_props.value.sub_yt_channels.sort()
    side_bar_props.value.sub_tw_channels.sort()
    utils.set_yt_id_list(side_bar_props.value.sub_yt_channels)
    utils.set_tw_id_list(side_bar_props.value.sub_tw_channels)
    update_video_events()
    show_popup('Imported ' + imported_count + ' channels')
  })
}

const YT_VIDEO_PATTERN =
  /^([\w\d_-]+|https:\/\/www.youtube.com\/watch\\?.*v=[\w\d_-]+.*|https:\/\/youtu.be\/[\w\d_-]+)$/
const is_youtube_video_url: ComputedRef<boolean> = computed(() =>
  YT_VIDEO_PATTERN.test(search_bar_val.value)
)

const YT_CHANNEL_PATTERN =
  /^([\w\d_-]+|https:\/\/(www.)?youtube.com\/@[\w\d_-]+|https:\/\/www.youtube.com\/channel\/[\w\d_-]+)$/
const is_youtube_channel_url: ComputedRef<boolean> = computed(() =>
  YT_CHANNEL_PATTERN.test(search_bar_val.value)
)

const TW_CHANNEL_PATTERN = /^([\d\w_]+|https:\/\/www.twitch.tv\/[\d\w_]+\/?)$/
const is_twitch_channel_url: ComputedRef<boolean> = computed(() =>
  TW_CHANNEL_PATTERN.test(search_bar_val.value)
)

const IMPORT_URL_PATTERN = new RegExp(
  utils.site_url.replace(utils.REGEXP_SPECIAL_CHAR, '\\$&') + 'cal?'
)
const is_import_url: ComputedRef<boolean> = computed(() =>
  IMPORT_URL_PATTERN.test(search_bar_val.value)
)

const is_search_bar_focused: Ref<boolean> = ref(false)
const search_bar_place_holder: ComputedRef<string> = computed(() => {
  if (is_search_bar_focused.value) {
    return 'Input ? for help'
  } else {
    return 'Search'
  }
})
const show_search_bar_help: ComputedRef<boolean> = computed(() => {
  return search_bar_val.value.trim() == '?' || search_bar_val.value.trim() == '？'
})
const help_lang: Ref<boolean> = ref(false)

function search_bar_focused() {
  const refresh_btn = document.getElementById('refresh_btn')
  const search_bar = document.getElementById('search_bar')
  if (refresh_btn == null || search_bar == null) {
    return
  }
  const search_bar_left = refresh_btn.offsetLeft + refresh_btn.offsetWidth + 40
  const width = window.innerWidth - search_bar_left * 2
  search_bar.style.width = width + 'px'
  is_search_bar_focused.value = true
}

function search_bar_unfocused() {
  const search_bar = document.getElementById('search_bar')
  if (search_bar == null) {
    return
  }
  search_bar.style.width = '4em'
  is_search_bar_focused.value = false
}
function search_bar_changed() {
  yt_channel_state.value = 'none'
  tw_channel_state.value = 'none'
  to_top()
}
document.getElementById('body')!.addEventListener('mousemove', utils.on_mouse_move)
document.getElementById('body')!.addEventListener('mousedown', utils.on_mouse_move)

function notice_yt_video() {
  utils.notice_yt_video(search_bar_val.value).then(
    (resp) => {
      show_popup(resp.result)
    },
    (reject) => {
      show_popup(reject)
    }
  )
}

function show_popup(msg: string) {
  popup_msgs.value.push({ msg: msg, time: Date.now() })
}

function to_top() {
  console.log('totop')
  window.scroll(0, 0)
}

document.getElementById('body')?.addEventListener('keyup', (event) => {
  if (event.key == 's' || event.key == 'S') {
    document.getElementById('search_bar')?.focus()
  }
  if (event.key == 'Escape') {
    search_bar_val.value = ''
    yt_channel_state.value = 'none'
    tw_channel_state.value = 'none'
  }
})
document.getElementById('body')?.addEventListener('touchstart', touch_start, false)
document.getElementById('body')?.addEventListener('touchmove', touch_move, false)

utils.set_sync_key(utils.get_sync_key())
utils.set_tw_id_list(utils.get_tw_id_list())
utils.set_yt_id_list(utils.get_yt_id_list())
update_video_events()
</script>

<template>
  <div id="menu_content" :style="sidebar_reactive_style">
    <SideBar
      v-bind="side_bar_props"
      @set_sync_key="set_sync_key"
      @unfollow_tw_ch="unfollow_tw_ch"
      @unfollow_yt_ch="unfollow_yt_ch"
      @clear_all_channel="clear_all_ch"
      @clear_ch_preview="clear_ch_preview"
      @import_list="import_list"
      @pull_sync_key="pull_sync_key"
      @show_popup="show_popup"
    ></SideBar>
  </div>
  <div
    class="header"
    style="background-color: #555; justify-content: center; pointer-events: all"
    @click="to_top"
  >
    <input
      type="text"
      v-model="search_bar_val"
      :placeholder="search_bar_place_holder"
      id="search_bar"
      @focus="search_bar_focused"
      @focusout="search_bar_unfocused"
      @keyup="search_bar_changed"
      autocomplete="off"
    />
  </div>
  <div class="header">
    <div class="header_btn">
      <input type="checkbox" id="menu_control" hidden="true" v-model="sidebar_control" />
      <label for="menu_control" style="display: flex">
        <img class="floating_btn_icon" src="/icons8-menu.svg" />
      </label>
    </div>
    <div
      id="refresh_btn"
      class="header_btn"
      @click="
        update_video_events().then(
          () => show_popup('Refreshed'),
          (msg) => show_popup(msg)
        )
      "
    >
      <label style="display: flex">
        <img class="floating_btn_icon" src="/icons8-refresh.svg" />
      </label>
    </div>
    <div style="flex: 1 1 auto; text-align: right; pointer-events: none">
      <div
        class="header_btn"
        id="cancel_btn"
        @click="search_bar_val = ''"
        style="pointer-events: all"
        :class="{ show: search_bar_val != '' }"
      >
        <button
          style="height: 1em; background-color: #0000; border: none; color: black; margin: 0px"
        >
          X
        </button>
      </div>
    </div>
  </div>
  <div class="header_floating_area">
    <div
      class="hdr_floating_btn"
      style="background-color: rgb(255, 3, 3)"
      :class="{
        hdr_floating_btn_show: is_youtube_channel_url,
        hdr_floating_btn_hidden: !is_youtube_channel_url,
        hdr_floating_btn_expand: yt_channel_state != 'none'
      }"
    >
      <span class="hdr_floating_btn" @click="load_youtube(search_bar_val)">Search Youtube</span>
      &nbsp;
      <span
        class="hdr_floating_btn"
        v-if="yt_channel_state != 'none'"
        @click="yt_channel_state = 'none'"
        >X</span
      >
      <ChannelProfile
        v-bind="yt_channel_data"
        :state="yt_channel_state"
        @follow="follow_youtube"
      ></ChannelProfile>
    </div>
    <div
      class="hdr_floating_btn"
      style="background-color: rgb(145, 71, 255)"
      :class="{
        hdr_floating_btn_show: is_twitch_channel_url,
        hdr_floating_btn_hidden: !is_twitch_channel_url,
        hdr_floating_btn_expand: tw_channel_state != 'none'
      }"
    >
      <span class="hdr_floating_btn" @click="load_twitch(search_bar_val)">Search Twitch</span>
      &nbsp;
      <span
        class="hdr_floating_btn"
        v-if="tw_channel_state != 'none'"
        @click="tw_channel_state = 'none'"
        >X</span
      >
      <ChannelProfile
        v-bind="tw_channel_data"
        :state="tw_channel_state"
        @follow="follow_twitch"
      ></ChannelProfile>
    </div>
    <div
      class="hdr_floating_btn"
      style="background-color: rgb(64, 157, 31)"
      :class="{
        hdr_floating_btn_show: is_youtube_video_url,
        hdr_floating_btn_hidden: !is_youtube_video_url
      }"
    >
      <span class="hdr_floating_btn" @click="notice_yt_video">Notice</span>
    </div>
    <div
      class="hdr_floating_btn"
      style="background-color: rgb(173, 41, 41)"
      :class="{
        hdr_floating_btn_show: is_import_url,
        hdr_floating_btn_hidden: !is_import_url
      }"
    >
      <span class="hdr_floating_btn" @click="import_list(search_bar_val)">Import List</span>
    </div>
    <div
      class="hdr_floating_btn"
      style="background-color: rgb(0, 142, 189); padding: 10px"
      :class="{
        hdr_floating_btn_show: show_search_bar_help,
        hdr_floating_btn_hidden: !show_search_bar_help,
        hdr_floating_btn_expand: show_search_bar_help
      }"
      @click="help_lang = !help_lang"
    >
      <span v-if="!help_lang" class="hdr_floating_btn"
        >接受的 Youtube 頻道格式:<br />
        https://www.youtube.com/@GawrGura <br />
        GawrGura<br />
        https://www.youtube.com/channel/UCoSrY_IQQVpmIRZ9Xf-y93g <br /><br />
        接受的 Twitch 頻道格式:<br />
        https://www.twitch.tv/restiafps <br />
        restiafps <br /><br />
        可以用側邊欄產生的行事曆網址一次匯入所有頻道:<br />
        https://li.paddycup1.idv.tw/cal?yt-ch=...&tw-ch=...<br /><br />
        讓 server "注意"到非公開 / 過於老舊的直播待機室:<br />
        https://www.youtube.com/watch?v=k7dTnCl2pVA<br />
        k7dTnCl2pVA<br />
        https://youtu.be/k7dTnCl2pVA<br /><br />
        Click for English / 點擊以顯示英文</span
      >
      <span v-if="help_lang" class="hdr_floating_btn"
        >To add youtube channel: https://www.youtube.com/@GawrGura <br />
        GawrGura<br />
        https://www.youtube.com/channel/UCoSrY_IQQVpmIRZ9Xf-y93g <br /><br />
        To add twitch channel:<br />
        https://www.twitch.tv/restiafps <br />
        restiafps <br /><br />
        To import list, copy calendar url produced by this tool:<br />
        https://li.paddycup1.idv.tw/cal?yt-ch=...&tw-ch=...<br /><br />
        To let the server "notice" a unlisted / old waiting room:<br />
        https://www.youtube.com/watch?v=k7dTnCl2pVA<br />
        k7dTnCl2pVA<br />
        https://youtu.be/k7dTnCl2pVA<br /><br />
        Click for Chinese / 點擊以顯示中文</span
      >
    </div>
  </div>
  <h2>
    Ongoing (<span v-if="ongoing_videos.length == filtered_ongoing_videos.length">{{
      ongoing_videos.length
    }}</span
    ><span v-if="ongoing_videos.length != filtered_ongoing_videos.length"
      >{{ filtered_ongoing_videos.length }} / {{ ongoing_videos.length }}</span
    >)
  </h2>
  <div class="video_container">
    <VideoComponent
      v-for="(video, index) in filtered_ongoing_videos"
      v-bind="video"
      :current_time="current_time"
      v-bind:key="index"
    ></VideoComponent>
  </div>

  <h2 v-if="starting_videos.length != 0">
    Starting (<span v-if="starting_videos.length == filtered_starting_videos.length">{{
      starting_videos.length
    }}</span
    ><span v-if="starting_videos.length != filtered_starting_videos.length"
      >{{ filtered_starting_videos.length }} / {{ starting_videos.length }}</span
    >)
  </h2>
  <div class="video_container" v-if="starting_videos.length != 0">
    <VideoComponent
      v-for="(video, index) in filtered_starting_videos"
      v-bind="video"
      :current_time="current_time"
      v-bind:key="index"
    ></VideoComponent>
  </div>

  <h2>
    Upcoming (<span v-if="upcoming_videos.length == filtered_upcoming_videos.length">{{
      upcoming_videos.length
    }}</span
    ><span v-if="upcoming_videos.length != filtered_upcoming_videos.length"
      >{{ filtered_upcoming_videos.length }} / {{ upcoming_videos.length }}</span
    >)
  </h2>
  <div class="video_container">
    <VideoComponent
      v-for="(video, index) in filtered_upcoming_videos"
      v-bind="video"
      :current_time="current_time"
      v-bind:key="index"
    ></VideoComponent>
  </div>
  <div id="footer"></div>
  <div
    id="menu_remaining_area"
    v-if="sidebar_control"
    @click="sidebar_control = !sidebar_control"
  ></div>
  <div id="popup_area">
    <div
      v-for="(item, idx) in popup_msgs"
      :key="idx"
      class="popup_msg"
      @click="item.time -= 2000"
      :class="{ fade_out: current_time.getTime() - item.time > 2000 }"
    >
      {{ item.msg }}
    </div>
  </div>
</template>

<style scoped>
div.video_container {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  width: 100%;
}

div#menu_content {
  position: fixed !important;
  top: 0px;
  left: 0px;
  height: 100%;
  max-width: var(--menu-width);
  z-index: 2;
  background-color: #333333;
  overflow-x: hidden;
  transition: width 0.1s;
  width: var(--menu-width);
  padding: 5px;
  overflow-y: scroll;
}

div#menu_remaining_area {
  width: 100vw;
  height: 100vh;
  position: fixed;
  top: 0px;
  left: 0px;
  z-index: 1;
  background-color: #00000088;
}

#floating_controls {
  position: fixed;
  top: 0px;
  left: 0px;
  margin: 10px;
  display: flex;
  flex-direction: row;
  z-index: 3;
}

div.header_btn {
  border-radius: 50%;
  background-color: ghostwhite;
  place-items: center;
  display: inline-block;
  flex-direction: column;
  vertical-align: middle;
  border: solid 2px slategray;
  margin-left: 7px;
  aspect-ratio: 1 /1;
  pointer-events: all;
}

div.header_btn:hover {
  background-color: rgb(219, 219, 219);
}

img.floating_btn_icon {
  width: 100%;
  height: 100%;
}
button#menu_close_btn {
  visibility: hidden;
}

div#footer {
  visibility: hidden;
}

input.error {
  border: solid red 2px;
}

div.header {
  height: 2em;
  width: 100%;
  position: fixed;
  top: 0;
  left: 0;
  padding-top: 10px;
  padding-bottom: 10px;
  z-index: 4;
  display: flex;
  pointer-events: none;
}

input#search_bar {
  width: 4em;
  transition: width 0.2s;
  border-radius: 2em;
  padding-top: 2px;
  padding-bottom: 2px;
  padding-left: 1em;
  border-style: none;
  pointer-events: all;
  z-index: 5;
}

div#search_bar_container {
  width: 5rem;
  transition: width 0.2s;
}
/* input#search_bar:focus { */
/* flex: 2 0 auto; */
/* } */

div.header_floating_area {
  width: 100%;
  position: fixed;
  top: 0;
  left: 0;
  display: flex;
  justify-content: center;
  flex-wrap: wrap;
  pointer-events: none;
  z-index: 3;
}

div.hdr_floating_btn {
  display: inline;
  transition: translate 1s;
  margin: 5px;
  height: 2em;
  border-radius: 2em;
  padding-left: 7px;
  padding-right: 7px;
  vertical-align: top;
  text-align: center;
  overflow: clip;
  pointer-events: all;
}

div.hdr_floating_btn_show {
  translate: 0 3.5em;
  max-width: 100%;
}

div.hdr_floating_btn_hidden {
  max-width: 0;
  transition: max-width 0.5s;
  visibility: hidden;
  user-select: none;
  pointer-events: none;
}

div.hdr_floating_btn_expand {
  height: auto;
}

span.hdr_floating_btn {
  vertical-align: middle;
  height: 2em;
}

div.hdr_floating_btn:hover {
  filter: brightness(1.2);
}

div.hdr_floating_btn:active {
  filter: brightness(1.4);
}

div#cancel_btn {
  translate: 100% 0;
  transition: translate 0.2s;
}

div#cancel_btn.show {
  translate: -2em 0;
}

div#popup_area {
  pointer-events: none;
  position: fixed;
  bottom: 0px;
  top: auto;
  right: 75%;
  left: 25%;
  width: 50%;
  z-index: 4;
}

div.popup_msg {
  opacity: 1;
  transition: opacity 0.2s;
  background-color: rgb(90, 90, 241);
  padding: 10px;
  border-radius: 8px;
  margin: 5px;
  pointer-events: all;
  text-align: center;
  width: auto;
}
div.popup_msg:hover {
  background-color: rgb(142, 142, 253);
}
div.popup_msg:active {
  background-color: rgb(142, 142, 253);
}

div.popup_msg.fade_out {
  opacity: 0;
}

@media (pointer: none), (pointer: coarse) {
  #floating_controls {
    position: fixed;
    top: auto;
    bottom: 0px;
    left: 0px;
    margin: 10px;
  }

  button#menu_close_btn {
    position: fixed;
    top: auto;
    bottom: 0px;
    left: 0px;
    margin: 10px;
  }

  div#footer {
    height: 2em;
    visibility: visible;
    padding: 10px 0px;
  }

  div.header {
    top: auto;
    bottom: 0px;
    pointer-events: none;
  }

  div.header_floating_area {
    top: auto;
    bottom: 0px;
    display: flex;
  }
  div.hdr_floating_btn_show {
    translate: 0 -3.5em;
  }

  div#popup_area {
    bottom: auto;
    top: 0px;
    transform: rotate(180deg);
  }
  div.popup_msg {
    transform: rotate(-180deg);
  }

  div.header {
    height: 2.5em;
  }
  span.hdr_floating_btn {
    height: 2.5em;
  }
}
</style>
