<script setup lang="ts">
import VideoComponent from './components/VideoComponent.vue'
import SideBar from './components/SideBar.vue'
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
let starting_videos: Ref<VideoEvent[]> = ref([])
let ongoing_videos: Ref<VideoEvent[]> = ref([])
let current_time: Ref<Date> = ref(new Date())
const sidebar_control = ref(false)
const sidebar_reactive_style: ComputedRef<StyleValue> = computed(() => {
  if (sidebar_control.value) {
    return { width: 'var(--menu-width)', padding: '5px', overflowY: 'scroll' }
  } else {
    return { width: '0px', padding: '0px', overflowY: 'hidden' }
  }
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

function load_youtube(query: string) {
  utils.load_youtube_channel(query).then((data) => {
    side_bar_props.value.yt_display_name = data.display
    side_bar_props.value.yt_handle = data.handle
    side_bar_props.value.yt_thumbnail = data.thumbnail
  })
}

function follow_youtube() {
  if (side_bar_props.value.yt_handle == null || side_bar_props.value.yt_handle.length == 0) {
    return
  }
  let handle = side_bar_props.value.yt_handle
  if (handle.startsWith('@')) {
    handle = handle.substring(1)
  }
  if (side_bar_props.value.sub_yt_channels.indexOf(handle) == -1) {
    side_bar_props.value.sub_yt_channels.push(handle)
    utils.set_yt_id_list(side_bar_props.value.sub_yt_channels)
    update_video_events()
  } else {
    console.log(side_bar_props.value.yt_handle + ' already followed')
  }
}

function load_twitch(query: string) {
  utils.load_twitch_channel(query).then((data) => {
    side_bar_props.value.tw_display_name = data.display
    side_bar_props.value.tw_handle = data.handle
    side_bar_props.value.tw_thumbnail = data.thumbnail
  })
}

function follow_twitch() {
  if (side_bar_props.value.tw_handle == null || side_bar_props.value.tw_handle.length == 0) {
    return
  }

  if (side_bar_props.value.sub_tw_channels.indexOf(side_bar_props.value.tw_handle) == -1) {
    side_bar_props.value.sub_tw_channels.push(side_bar_props.value.tw_handle)
    utils.set_tw_id_list(side_bar_props.value.sub_tw_channels)
    update_video_events()
  } else {
    console.log(side_bar_props.value.tw_handle + ' already followed')
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
    utils.set_yt_id_list(side_bar_props.value.sub_yt_channels)
    for (const ch of resp.tw_ch) {
      if (side_bar_props.value.sub_tw_channels.indexOf(ch) == -1) {
        side_bar_props.value.sub_tw_channels.push(ch)
      }
    }
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
          if (side_bar_props.value.sub_yt_channels.indexOf(info.handle) == -1) {
            side_bar_props.value.sub_yt_channels.push(info.handle)
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
    utils.set_yt_id_list(side_bar_props.value.sub_yt_channels)
    utils.set_tw_id_list(side_bar_props.value.sub_tw_channels)
    update_video_events()
  })
}

update_video_events()
</script>

<template>
  <div id="floating_controls">
    <div class="floating_btn">
      <input type="checkbox" id="menu_control" hidden="true" v-model="sidebar_control" />
      <label for="menu_control" style="display: flex">
        <img class="floating_btn_icon" src="/icons8-menu.svg" />
      </label>
    </div>
    <div class="floating_btn" @click="update_video_events">
      <label style="display: flex">
        <img class="floating_btn_icon" src="/icons8-refresh.svg" />
      </label>
    </div>
  </div>
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

  <h2>Ongoing ({{ ongoing_videos.length }})</h2>
  <div class="video_container">
    <VideoComponent
      v-for="(video, index) in ongoing_videos"
      v-bind="video"
      :current_time="current_time"
      v-bind:key="index"
    ></VideoComponent>
  </div>

  <h2 v-if="starting_videos.length != 0">Starting ({{ starting_videos.length }})</h2>
  <div class="video_container" v-if="starting_videos.length != 0">
    <VideoComponent
      v-for="(video, index) in starting_videos"
      v-bind="video"
      :current_time="current_time"
      v-bind:key="index"
    ></VideoComponent>
  </div>

  <h2>Upcoming ({{ upcoming_videos.length }})</h2>
  <div class="video_container">
    <VideoComponent
      v-for="(video, index) in upcoming_videos"
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

div.floating_btn {
  border-radius: 50%;
  background-color: ghostwhite;
  place-items: center;
  display: flex;
  flex-direction: column;
  vertical-align: middle;
  border: solid 2px slategray;
  margin-right: 10px;
}

div.floating_btn:hover {
  background-color: rgb(219, 219, 219);
}

.floating_btn_icon {
  margin: 2px;
  width: 2em;
  height: 2em;
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
    visibility: visable;
  }
}
</style>
