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

setInterval(
  () => {
    update_video_events()
  },
  1000 * 60 * 5
)
setInterval(() => {
  current_time.value = new Date()
}, 1000)

side_bar_props.value.sub_tw_channels.sort()
side_bar_props.value.sub_yt_channels.sort()

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
  } else {
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
  } else {
    console.log(tw_channel_data.value.handle + ' already followed')
  }
}

function update_video_events() {
  const new_ongoing_videos: VideoEvent[] = []
  const new_starting_videos: VideoEvent[] = []
  const new_upcoming_videos: VideoEvent[] = []
  console.log('Updating video list')
  utils
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
  utils.pull_sync_key(side_bar_props.value.sync_key).then((resp) => {
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
  })
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
  })
}

const YT_VIDEO_PATTERN = /^([\w\d_-]+|https:\/\/www.youtube.com\/watch\\?.*v=[\w\d_-]+.*)$/
const is_youtube_video_url: ComputedRef<boolean> = computed(() =>
  YT_VIDEO_PATTERN.test(search_bar_val.value)
)

const YT_CHANNEL_PATTERN =
  /^([\w\d_-]+|https:\/\/www.youtube.com\/@[\w\d_-]+|https:\/\/www.youtube.com\/channel\/[\w\d_-]+)$/
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
console.log(utils.site_url.replace(utils.REGEXP_SPECIAL_CHAR, '\\$&') + 'cal?')
const is_import_url: ComputedRef<boolean> = computed(() =>
  IMPORT_URL_PATTERN.test(search_bar_val.value)
)

function search_bar_focused() {
  const refresh_btn = document.getElementById('refresh_btn')
  const search_bar = document.getElementById('search_bar')
  if (refresh_btn == null || search_bar == null) {
    return
  }
  const search_bar_left = refresh_btn.offsetLeft + refresh_btn.offsetWidth + 10
  const width = window.innerWidth - search_bar_left * 2
  search_bar.style.width = width + 'px'
}

function search_bar_unfocused() {
  const search_bar = document.getElementById('search_bar')
  if (search_bar == null) {
    return
  }
  search_bar.style.width = '5em'
}
function search_bar_changed() {
  yt_channel_state.value = 'none'
  tw_channel_state.value = 'none'
}

update_video_events()
</script>

<template>
  <div id="menu_content" :style="sidebar_reactive_style">
    <SideBar
      v-bind="side_bar_props"
      @load-youtube="load_youtube"
      @follow-youtube="follow_youtube"
      @load-twitch="load_twitch"
      @follow-twitch="follow_twitch"
      @set_sync_key="set_sync_key"
      @unfollow_tw_ch="unfollow_tw_ch"
      @unfollow_yt_ch="unfollow_yt_ch"
      @clear_all_channel="clear_all_ch"
      @clear_ch_preview="clear_ch_preview"
      @import_list="import_list"
      @pull_sync_key="pull_sync_key"
    ></SideBar>
  </div>
  <div class="header" style="background-color: #555; justify-content: center">
    <input
      type="text"
      v-model="search_bar_val"
      placeholder="Search"
      id="search_bar"
      @focus="search_bar_focused"
      @focusout="search_bar_unfocused"
      @keyup="search_bar_changed"
    />
  </div>
  <div class="header">
    <div class="header_btn">
      <input type="checkbox" id="menu_control" hidden="true" v-model="sidebar_control" />
      <label for="menu_control" style="display: flex">
        <img class="floating_btn_icon" src="/icons8-menu.svg" />
      </label>
    </div>
    <div id="refresh_btn" class="header_btn" @click="update_video_events">
      <label style="display: flex">
        <img class="floating_btn_icon" src="/icons8-refresh.svg" />
      </label>
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
      <span class="hdr_floating_btn" @click="utils.notice_yt_video(search_bar_val)">Notice</span>
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
  <div
    id="menu_remaining_area"
    v-if="sidebar_control"
    @click="sidebar_control = !sidebar_control"
  ></div>
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

div#menu_footer {
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
  z-index: 3;
  display: flex;
  user-select: none;
  pointer-events: none;
}

div.header > * {
  pointer-events: all;
}

input#search_bar {
  margin-left: 1em;
  width: 5em;
  transition: width 0.2s;
  border-radius: 2em;
  padding-top: 2px;
  padding-bottom: 2px;
  padding-left: 1em;
  flex: 0 1 auto;
  border-style: none;
  pointer-events: all;
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

  div#menu_footer {
    height: 30px;
    visibility: visible;
  }
}
</style>
