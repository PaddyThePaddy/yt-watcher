export const REGEXP_SPECIAL_CHAR = /[!#$%^&*)(+=.<>{}[\]:;'"|~`_-]/g
export let api_url = 'https://' + window.location.hostname + '/api/'

export type UpcomingEvent = {
  start_date_time: string
  start_timestamp_millis: number
  thumbnail_url: string
  title: string
  description: string
  target_url: string
  ongoing: boolean
  source: {
    YoutubeChannel?: {
      id: string
      thumbnail_url: string
      title: string
      custom_url: string
    }
    TwitchChannel?: {
      id: string
      thumbnail_url: string
      title: string
      login: string
    }
  }
  uid: string
}

export const mouse_pos = { x: 0, y: 0 }
export function on_mouse_move(e: MouseEvent) {
  mouse_pos.x = e.x
  mouse_pos.y = e.y
}

function set_cookie(name: string, value: string | null, max_age: number | null) {
  let maxAge = ''
  if (max_age) {
    maxAge = '; max-age=' + max_age
  }
  document.cookie = name + '=' + (value || '') + maxAge + '; path=/'
}

export function get_yt_id_list(): string[] {
  const cookies: string[] = document.cookie.split(';')
  for (const c of cookies) {
    const pair = c.split('=', 2)
    if (pair[0].trim() == 'yt_id_list') {
      return pair[1]
        .trim()
        .split(',')
        .filter((s) => s.length != 0)
    }
  }
  return []
}

export function set_yt_id_list(id_list: string[]) {
  set_cookie('yt_id_list', id_list.join(','), 31536000)
}

export function get_tw_id_list(): string[] {
  const cookies: string[] = document.cookie.split(';')
  for (const c of cookies) {
    const pair = c.split('=', 2)
    if (pair[0].trim() == 'tw_id_list') {
      return pair[1]
        .trim()
        .split(',')
        .filter((s) => s.length != 0)
    }
  }
  return []
}

export function set_tw_id_list(id_list: string[]) {
  set_cookie('tw_id_list', id_list.join(','), 31536000)
}

export function get_sync_key() {
  const cookies = document.cookie.split(';')
  for (const c of cookies) {
    const pair = c.split('=', 2)
    if (pair[0].trim() == 'sync_key') {
      return pair[1].trim()
    }
  }
  return ''
}

export function set_sync_key(key: string) {
  set_cookie('sync_key', key, 31536000)
}

export type ChannelInfo = {
  display: string
  handle: string
  thumbnail: string
}

export function load_youtube_channel(query: string): Promise<ChannelInfo> {
  return fetch(api_url + 'yt-ch?q=' + query)
    .then((resp) => resp.json())
    .then((resp: any) => {
      return new Promise((resolve, reject) => {
        if (resp.error == null) {
          resolve({
            display: resp.data.title,
            handle: resp.data.custom_url,
            thumbnail: resp.data.thumbnail
          })
        } else {
          reject(resp.error)
        }
      })
    })
}

export function load_twitch_channel(query: string): Promise<ChannelInfo> {
  return fetch(api_url + 'tw-ch?q=' + query)
    .then((resp) => resp.json())
    .then((resp: any) => {
      return new Promise((resolve, reject) => {
        if (resp.error == null) {
          resolve({
            display: resp.data.title,
            handle: resp.data.custom_url,
            thumbnail: resp.data.thumbnail
          })
        } else {
          reject(resp.error)
        }
      })
    })
}

export function get_video_data(
  yt_ch_list: string[],
  tw_ch_list: string[]
): Promise<UpcomingEvent[]> {
  if (yt_ch_list.length == 0 && tw_ch_list.length == 0) {
    return new Promise((_, reject) => {
      reject('No tracking channel')
    })
  }
  let url = api_url + 'data?'
  if (yt_ch_list.length != 0) {
    url += 'yt-ch=' + yt_ch_list.join(',')
  }
  if (tw_ch_list.length != 0) {
    url += '&tw-ch=' + tw_ch_list.join(',')
  }
  return fetch(url)
    .then((resp) => resp.json())
    .then((resp) => {
      return resp
    })
}

export function verify_sync_key(key: string): boolean {
  return /^[\w\d]{8}-[\w\d]{4}-[\w\d]{4}-[\w\d]{4}-[\w\d]{12}$/.test(key)
}

export function new_sync_key(): Promise<string | undefined | void> {
  return fetch(api_url + 'sync/new')
    .then((resp) => resp.json())
    .then((resp) => {
      return resp.key
    })
}

export function push_sync_key(sync_key: string, yt_ch: string[], tw_ch: string[]) {
  if (!verify_sync_key(sync_key)) {
    console.log('invalid sync key')
    return new Promise((_, reject) => {
      reject('Invalid sync key')
    })
  }
  let url = api_url + 'sync/push?key=' + sync_key
  if (yt_ch.length != 0) {
    url += '&yt-ch=' + yt_ch.join(',')
  }
  if (tw_ch.length != 0) {
    url += '&tw-ch=' + tw_ch.join(',')
  }
  return fetch(url)
    .then((resp) => resp.json())
    .then((resp) => {
      console.log(resp)
      return resp
    })
}

export function pull_sync_key(sync_key: string): Promise<{ yt_ch: string[]; tw_ch: string[] }> {
  if (!verify_sync_key(sync_key)) {
    console.log('invalid sync key')
    return new Promise((_, reject) => reject())
  }
  return fetch(api_url + 'sync/pull?key=' + get_sync_key())
    .then((resp) => {
      return resp.json()
    })
    .then((resp) => {
      const ret = { yt_ch: [], tw_ch: [] }
      if (resp.yt_ch != null) {
        ret.yt_ch = ret.yt_ch.concat(resp.yt_ch)
      }
      if (resp.tw_ch != null) {
        ret.tw_ch = ret.tw_ch.concat(resp.tw_ch)
      }
      return ret
    })
}

export function notice_yt_video(value: string) {
  const url_pattern = new RegExp(
    '(?:https://www.youtube.com/watch\\?.*v=|https://youtu.be/)([\\w\\d_-]+)'
  )
  const id_list = []
  for (const s of value.split(',')) {
    const match = url_pattern.exec(s)
    if (match != null) {
      id_list.push(match[1])
    } else {
      id_list.push(s)
    }
  }
  const url = api_url + 'notice-yt-video?id=' + id_list
  return fetch(url)
    .then((resp) => resp.json())
    .then((resp) => {
      console.log(resp)
      return resp
    })
}
