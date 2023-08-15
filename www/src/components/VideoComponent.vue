<script setup lang="ts">
const DAY_MS = 1000 * 60 * 60 * 24
const HOUR_MS = 1000 * 60 * 60
const MIN_MS = 1000 * 60
const prop = defineProps({
  target_url: String,
  thumbnail_url: String,
  title: String,
  start_time: {
    type: Date,
    required: true
  },
  current_time: {
    type: Date,
    required: true
  },
  ongoing: Boolean,
  source_name: String,
  source_profile_img_url: String,
  source_url: String,
  source_type: String
})

function get_time_delta_string(date: Date) {
  let before_after = ''
  if (date > prop.current_time) {
    before_after = ' later'
  } else {
    before_after = ' ago'
  }
  const delta_ms = Math.abs(date.getTime() - prop.current_time.getTime())
  if (delta_ms > DAY_MS) {
    return '' + Math.floor(delta_ms / DAY_MS) + ' days ' + before_after
  } else if (delta_ms > HOUR_MS) {
    return '' + Math.floor(delta_ms / HOUR_MS) + ' hours ' + before_after
  } else {
    return '' + Math.floor(delta_ms / MIN_MS) + ' mins ' + before_after
  }
}
</script>

<style scoped>
@import url('https://fonts.googleapis.com/css2?family=Noto+Sans+TC&display=swap');

div.video_card {
  margin: 0.5rem;
  padding: 0.5em;
  border-radius: 8px;
  transition: background-color 0.1s;
  background-clip: padding-box;
  background: var(--background-color);
}

div.video_card:hover {
  background-color: #333333;
}

div.twitch {
  border: solid rgb(145, 71, 255, 0.3);
}

div.youtube {
  border: solid rgba(255, 56, 56, 0.3);
}

div.video_title {
  overflow-wrap: break-word;
  max-height: 3rem;
  text-overflow: ellipsis;
  text-rendering: optimizelegibility;
  overflow: hidden;
  hyphens: auto;
  text-overflow: ellipsis;
  text-wrap: wrap;
  -webkit-line-clamp: 2;
  display: -webkit-box;
  word-break: break-word;
  -webkit-box-orient: vertical;
}

a {
  color: #f3f2ff;
  text-decoration: none;
}

img.channel_icon {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  flex-grow: 1;
}

div.card_lower_part {
  display: flex;
  max-width: 320px;
  width: 320px;
  flex-direction: row;
}

div.text_div {
  display: flex;
  max-width: 280px;
  flex-direction: column;
  padding-right: 22px;
  padding-left: 10px;
}

div.icon_div > a {
  display: flex;
  flex-direction: row;
  height: 100%;
  align-items: center;
}

span.channel_name {
  color: rgb(145, 145, 255);
}

img.video_thumbnail {
  border-radius: 5px;
  overflow-clip-margin: content-box;
  overflow: clip;
  width: 320px;
  height: 180px;
  transition: transform 0.3s;
}

@media (pointer: fine) {
  img.video_thumbnail {
    transition: transform 0.3s;
  }
  img.video_thumbnail:hover {
    transform: scale(2);
  }
}
</style>

<template>
  <div class="video_card" :class="prop.source_type">
    <a :href="prop.target_url">
      <img class="video_thumbnail" :src="prop.thumbnail_url" />
      <br />
      <div class="card_lower_part">
        <div class="icon_div">
          <a :href="prop.source_url">
            <img class="channel_icon" :src="prop.source_profile_img_url" />
          </a>
        </div>
        <div class="text_div">
          <div class="video_title" :title="prop.title">{{ prop.title }}</div>
          <a :href="prop.source_url">
            <span class="channel_name" :prop="prop.source_name">{{ prop.source_name }}</span>
          </a>
          <span class="video_time" :title="prop.start_time.toLocaleString()">{{
            get_time_delta_string(prop.start_time!)
          }}</span>
        </div>
      </div>
    </a>
  </div>
</template>
