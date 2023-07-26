const DAY_MS = 1000 * 60 * 60 * 24;
const HOUR_MS = 1000 * 60 * 60;
const MIN_MS = 1000 * 60;

let site_url = document.URL;
let yt_channel_data = null;
let tw_channel_data = null;
let video_data = null;
if (site_url.endsWith("index.html")) {
  site_url = site_url.substring(0, site_url.length - 10);
}
if (!site_url.endsWith("/")) {
  site_url += "/";
}

setInterval(() => {
  update_video_list();
}, MIN_MS * 5);

setInterval(() => {
  render_video_list();
}, MIN_MS);

function load() {
  update_video_list();
  update_channel_id_list();
}

function get_yt_id_list() {
  const cookies = document.cookie.split(";");
  for (c of cookies) {
    const pair = c.split("=", 2);
    if (pair[0].trim() == "yt_id_list") {
      return pair[1].trim();
    }
  }
  return "";
}

function set_yt_id_list(id_list) {
  document.cookie = "yt_id_list=" + id_list;
}

function get_tw_id_list() {
  const cookies = document.cookie.split(";");
  for (c of cookies) {
    const pair = c.split("=", 2);
    if (pair[0].trim() == "tw_id_list") {
      return pair[1].trim();
    }
  }
  return "";
}

function set_tw_id_list(id_list) {
  document.cookie = "tw_id_list=" + id_list;
}

function load_yt_channel(value) {
  return fetch(site_url + "yt-ch?q=" + value)
    .then((resp) => {
      return resp.json();
    })
    .then((resp) => {
      if (resp.error == null) {
        yt_channel_data = resp.data;
      }
    });
}

function load_tw_channel(value) {
  return fetch(site_url + "tw-ch?q=" + value)
    .then((resp) => {
      return resp.json();
    })
    .then((resp) => {
      if (resp.error == null) {
        tw_channel_data = resp.data;
      }
    });
}

function build_yt_channel_div(title, url, thumbnail_url) {
  const frame = document.createElement("div");
  const link = document.createElement("a");
  link.href = "https://www.youtube.com/" + url;
  const img = document.createElement("img");
  img.src = thumbnail_url;
  img.classList = "channel_avatar";
  const title_ele = document.createElement("span");
  title_ele.innerHTML = title;
  const url_ele = document.createElement("span");
  url_ele.innerHTML = url;
  link.appendChild(img);
  link.appendChild(document.createElement("br"));
  link.appendChild(title_ele);
  link.appendChild(document.createElement("br"));
  link.appendChild(url_ele);
  frame.appendChild(link);
  return frame;
}

function build_tw_channel_div(title, login, thumbnail_url) {
  const frame = document.createElement("div");
  const link = document.createElement("a");
  link.href = "https://www.twitch.tv/" + login;
  const img = document.createElement("img");
  img.src = thumbnail_url;
  img.classList = "channel_avatar";
  const title_ele = document.createElement("span");
  title_ele.innerHTML = title;
  const login_ele = document.createElement("span");
  login_ele.innerHTML = login;
  link.appendChild(img);
  link.appendChild(document.createElement("br"));
  link.appendChild(title_ele);
  link.appendChild(document.createElement("br"));
  link.appendChild(login_ele);
  frame.appendChild(link);
  return frame;
}

function check_yt_channel() {
  load_yt_channel(document.getElementById("channel_name").value).then(() => {
    if (yt_channel_data != null) {
      let preview = build_yt_channel_div(
        yt_channel_data.title,
        yt_channel_data.custom_url,
        yt_channel_data.thumbnail
      );
      let preview_frame = document.getElementById("yt_channel_preview");
      while (preview_frame.childElementCount != 0) {
        preview_frame.removeChild(preview_frame.firstChild);
      }
      preview_frame.appendChild(preview);
      document.getElementById("follow_yt_btn").hidden = false;
      document.getElementById("check_yt_btn").hidden = true;
    } else {
      document.getElementById("follow_yt_btn").hidden = true;
      document.getElementById("check_yt_btn").hidden = false;
      clear_yt_id();
    }
  });
}

function check_tw_channel() {
  load_tw_channel(document.getElementById("channel_name").value).then(() => {
    if (tw_channel_data != null) {
      let preview = build_yt_channel_div(
        tw_channel_data.title,
        tw_channel_data.custom_url,
        tw_channel_data.thumbnail
      );
      let preview_frame = document.getElementById("tw_channel_preview");
      while (preview_frame.childElementCount != 0) {
        preview_frame.removeChild(preview_frame.firstChild);
      }
      preview_frame.appendChild(preview);
      document.getElementById("follow_tw_btn").hidden = false;
      document.getElementById("check_tw_btn").hidden = true;
    } else {
      document.getElementById("follow_tw_btn").hidden = true;
      document.getElementById("check_tw_btn").hidden = false;
      clear_tw_id();
    }
  });
}

function import_channel_list() {
  let list = document.getElementById("channel_name").value;
  let promises = [];
  const yt_list = new RegExp(".+(?:\\?|&)yt-ch=([^&]+)").exec(list);
  if (yt_list != null) {
    for (c of yt_list[1].split(",").filter((s) => s.length != 0)) {
      promises.push(
        load_yt_channel(c).then(() => {
          follow_yt_channel();
        })
      );
    }
  }
  const tw_list = new RegExp(".+(?:\\?|&)tw-ch=([^&]+)").exec(list);
  if (tw_list != null) {
    for (c of tw_list[1].split(",").filter((s) => s.length != 0)) {
      promises.push(
        load_tw_channel(c).then(() => {
          follow_tw_channel();
        })
      );
    }
  }
  Promise.all(promises).then(() => {
    update_video_list();
  });
}

function follow_yt_btn() {
  follow_yt_channel();
  update_video_list();
}

function follow_tw_btn() {
  follow_tw_channel();
  update_video_list();
}

function follow_yt_channel() {
  if (!yt_channel_data) {
    console.log("url is not valid");
    return;
  }
  let exist = false;
  let id_list = get_yt_id_list();
  for (id of id_list.split(",").filter((s) => s.length != 0)) {
    if (!id.startsWith("@")) {
      id = "@" + id;
    }
    if (id == yt_channel_data.custom_url) {
      exist = true;
      break;
    }
  }
  if (!exist) {
    if (id_list.length != 0) {
      id_list += ",";
    }
    let url = yt_channel_data.custom_url;
    if (url.startsWith("@")) {
      url = url.substring(1);
    }
    id_list += url;
    set_yt_id_list(id_list);
    update_channel_id_list();
  }
}

function follow_tw_channel() {
  if (!tw_channel_data) {
    console.log("url is not valid");
    return;
  }
  let exist = false;
  let id_list = get_tw_id_list();
  for (id of id_list.split(",").filter((s) => s.length != 0)) {
    if (!id.startsWith("@")) {
      id = "@" + id;
    }
    if (id == tw_channel_data.custom_url) {
      exist = true;
      break;
    }
  }
  if (!exist) {
    if (id_list.length != 0) {
      id_list += ",";
    }
    let url = tw_channel_data.custom_url;
    if (url.startsWith("@")) {
      url = url.substring(1);
    }
    id_list += url;
    set_tw_id_list(id_list);
    update_channel_id_list();
    update_video_list();
  }
}

function clear_id() {
  clear_tw_id();
  clear_yt_id();
}

function clear_yt_id() {
  let preview_frame = document.getElementById("yt_channel_preview");
  while (preview_frame.childElementCount != 0) {
    preview_frame.removeChild(preview_frame.firstChild);
  }
  yt_channel_data = null;
  document.getElementById("follow_yt_btn").hidden = true;
  document.getElementById("check_yt_btn").hidden = false;
}
function clear_tw_id() {
  let preview_frame = document.getElementById("tw_channel_preview");
  while (preview_frame.childElementCount != 0) {
    preview_frame.removeChild(preview_frame.firstChild);
  }
  yt_channel_data = null;
  document.getElementById("follow_tw_btn").hidden = true;
  document.getElementById("check_tw_btn").hidden = false;
}

function update_video_list() {
  const yt_id_list = get_yt_id_list()
    .split(",")
    .filter((s) => s.length != 0);
  const tw_id_list = get_tw_id_list()
    .split(",")
    .filter((s) => s.length != 0);
  console.log(new Date() + " updating video info");
  console.log(tw_id_list);
  if (yt_id_list.length == 0 && tw_id_list.length == 0) {
    console.log("id list is empty");
    video_data = null;
    render_video_list();
    return;
  }
  let url = site_url + "data?";
  if (yt_id_list.length != 0) {
    url += "yt-ch=" + yt_id_list.join(",");
  }
  if (tw_id_list.length != 0) {
    if (yt_id_list.length != 0) {
      url += "&";
    }
    url += "tw-ch=" + tw_id_list.join(",");
  }
  console.log(url);
  fetch(url)
    .then((resp) => {
      return resp.json();
    })
    .then((data) => (video_data = data))
    .then(() => {
      render_video_list();
    });
}

function render_video_list() {
  console.log(new Date() + " rendering video list");
  const ongoing_video_frame = document.getElementById("ongoing_video_frame");
  const upcoming_video_frame = document.getElementById("upcoming_video_frame");
  const starting_video_frame = document.getElementById("starting_video_frame");
  while (ongoing_video_frame.childElementCount != 0) {
    ongoing_video_frame.removeChild(ongoing_video_frame.firstChild);
  }
  while (upcoming_video_frame.childElementCount != 0) {
    upcoming_video_frame.removeChild(upcoming_video_frame.firstChild);
  }
  while (starting_video_frame.childElementCount != 0) {
    starting_video_frame.removeChild(starting_video_frame.firstChild);
  }

  if (video_data == null) {
    console.log("video_data is empty");
    return;
  }
  let ongoing_count = 0;
  let upcoming_count = 0;
  let starting_count = 0;
  for (data of video_data) {
    if (data.ongoing) {
      ongoing_video_frame.appendChild(build_video_preview(data));
      ongoing_count += 1;
    } else {
      if (data.start_timestamp_millis < Date.now()) {
        starting_video_frame.appendChild(build_video_preview(data));
        starting_count += 1;
      } else {
        upcoming_video_frame.appendChild(build_video_preview(data));
        upcoming_count += 1;
      }
    }
  }
  document.getElementById(
    "ongoing_header"
  ).innerHTML = `Ongoing Streams (${ongoing_count})`;
  document.getElementById(
    "upcoming_header"
  ).innerHTML = `Upcoming Streams (${upcoming_count})`;
  if (starting_count != 0) {
    const header = document.getElementById(
      "starting_header"
    );
    header.innerHTML = `Starting Streams (${starting_count})`;
    header.hidden = false;
    starting_video_frame.hidden = false;
  } else {
    document.getElementById(
      "starting_header"
    ).hidden=true;
    starting_video_frame.hidden = true;
  }
}

function build_video_preview(data) {
  const frame = document.createElement("div");
  const link = document.createElement("a");
  link.href = data.target_url;
  const img = document.createElement("img");
  img.src = data.thumbnail_url;
  img.classList.add("video_thumbnail");
  const title = document.createElement("div");
  title.innerHTML = data.title;
  title.classList = "video_title";
  title.title = data.title;
  const start_time = new Date(data.start_timestamp_millis);
  const time_span = document.createElement("span");
  time_span.innerHTML = get_time_delta_string(start_time);
  time_span.title = start_time.toLocaleString();
  const lower_part_div = document.createElement("div");
  lower_part_div.classList.add("lower_part_div");
  const icon_div = document.createElement("div");
  icon_div.classList.add("icon_div");
  const text_div = document.createElement("div");
  text_div.classList.add("text_div");
  const channel_icon = document.createElement("img");
  channel_icon.classList.add("channel_icon");
  const channel_name = document.createElement("span");
  channel_name.classList.add("channel_name");
  const channel_text_anchor = document.createElement("a");
  const channel_icon_anchor = document.createElement("a");

  if (data.source.YoutubeChannel) {
    channel_icon.src = data.source.YoutubeChannel.thumbnail_url;
    channel_name.innerHTML = data.source.YoutubeChannel.title;
    channel_text_anchor.href =
      "https://www.youtube.com/" + data.source.YoutubeChannel.custom_url;
    channel_icon_anchor.href =
      "https://www.youtube.com/" + data.source.YoutubeChannel.custom_url;
    frame.classList.add("youtube_video_card");
  } else if (data.source.TwitchChannel) {
    channel_icon.src = data.source.TwitchChannel.thumbnail_url;
    channel_name.innerHTML = data.source.TwitchChannel.title;
    console.log(data);
    channel_text_anchor.href =
      "https://www.twitch.tv/" + data.source.TwitchChannel.login;
    channel_icon_anchor.href =
      "https://www.twitch.tv/" + data.source.TwitchChannel.login;
    frame.classList.add("twitch_video_card");
  }

  channel_icon_anchor.appendChild(channel_icon);
  icon_div.appendChild(channel_icon_anchor);
  channel_text_anchor.appendChild(channel_name);
  text_div.appendChild(title);
  text_div.appendChild(channel_text_anchor);
  text_div.appendChild(time_span);
  lower_part_div.appendChild(icon_div);
  lower_part_div.appendChild(text_div);
  link.appendChild(img);
  link.appendChild(document.createElement("br"));
  link.appendChild(lower_part_div);
  frame.appendChild(link);
  frame.classList.add("video_frame");

  return frame;
}

function get_time_delta_string(date) {
  const now = new Date();
  let before_after = "";
  if (date > now) {
    before_after = " later";
  } else {
    before_after = " ago";
  }
  const delta_ms = Math.abs(date - now);
  if (delta_ms > DAY_MS) {
    return "" + Math.floor(delta_ms / DAY_MS) + " days " + before_after;
  } else if (delta_ms > HOUR_MS) {
    return "" + Math.floor(delta_ms / HOUR_MS) + " hours " + before_after;
  } else {
    return "" + Math.floor(delta_ms / MIN_MS) + " mins " + before_after;
  }
}

let copy_timeout_handle = null;
function copy_calendar_url() {
  const yt_id_list = get_yt_id_list()
    .split(",")
    .filter((s) => s.length != 0);
  const tw_id_list = get_tw_id_list()
    .split(",")
    .filter((s) => s.length != 0);
  if (yt_id_list.length == 0 && tw_id_list.length == 0) {
    console.log("No id for calendar to copy");
    return;
  }
  let url = site_url + "cal?";
  if (yt_id_list.length != 0) {
    url += "yt-ch=" + yt_id_list.join(",");
  }
  if (tw_id_list.length != 0) {
    if (yt_id_list.length != 0) {
      url += "&";
    }
    url += "tw-ch=" + tw_id_list.join(",");
  }
  if (document.getElementById("alarm_cb").checked) {
    url += "&alram=true";
  }
  navigator.clipboard.writeText(url).then(() => {
    console.log("copy success");
    document.getElementById("copy_popup").classList.add("show");
    if (copy_timeout_handle) {
      clearTimeout(copy_timeout_handle);
    }
    copy_timeout_handle = setTimeout(() => {
      document.getElementById("copy_popup").classList.remove("show");
      copy_timeout_handle = null;
    }, 2000);
  });
}
function open_menu() {
  const menu = document.getElementById("menu_content");
  menu.style.width = "var(--menu-width)";
  menu.style.padding = "5px";
  menu.style.overflow = "scroll";
  document.getElementById("menu_remaining_area").style.visibility = "visible";
}

function close_menu() {
  const menu = document.getElementById("menu_content");
  menu.style.padding = "0px";
  menu.style.overflow = "hidden";
  menu.style.width = "0px";
  document.getElementById("menu_remaining_area").style.visibility = "hidden";
}

function update_channel_id_list() {
  const yt_channel_id_list = get_yt_id_list()
    .split(",")
    .filter((s) => s.length != 0);
  const tw_channel_id_list = get_tw_id_list()
    .split(",")
    .filter((s) => s.length != 0);
  const id_list_parent = document.getElementById("channel_id_list");
  let count = 0;
  while (id_list_parent.childElementCount != 0) {
    id_list_parent.removeChild(id_list_parent.firstChild);
  }
  for (id of yt_channel_id_list) {
    if (id.trim().length > 0) {
      count += 1;
      id_list_parent.appendChild(build_channel_id_item(id + "@YT"));
    }
  }
  for (id of tw_channel_id_list) {
    if (id.trim().length > 0) {
      count += 1;
      id_list_parent.appendChild(build_channel_id_item(id + "@TW"));
    }
  }
  document.getElementById("channel_count").innerHTML =
    "Tracking " + count + " channels";
}

function build_channel_id_item(id) {
  const item = document.createElement("li");
  const id_span = document.createElement("span");
  const del_btn = document.createElement("button");
  del_btn.innerHTML = "&times";
  del_btn.onclick = () => {
    del_channel(id);
  };
  let display_id = id;
  if (display_id.endsWith("@YT")) {
    display_id = display_id.substring(0, display_id.length - 3);
    display_id =
      display_id +
      ' <a href="https://www.youtube.com/@' +
      display_id +
      '"><img src="assets/youtube_32x32.png" style="height: 1.2em; vertical-align: middle;"></a>';
  } else {
    display_id = display_id.substring(0, display_id.length - 3);
    display_id =
      display_id +
      ' <a href="https://www.twitch.tv/' +
      display_id +
      '"><img src="assets/twitch_32x32.png" style="height: 1.2em; vertical-align: middle;"></a>';
  }
  id_span.innerHTML = "@" + display_id;
  item.appendChild(del_btn);
  item.appendChild(id_span);
  return item;
}

function del_channel(target) {
  let current_id_list;
  console.log("Deleting " + target);
  let source;
  if (target.endsWith("@YT")) {
    source = "yt";
  } else {
    source = "tw";
  }
  if (source == "yt") {
    current_id_list = get_yt_id_list()
      .split(",")
      .filter((s) => s.length != 0);
  } else {
    current_id_list = get_tw_id_list()
      .split(",")
      .filter((s) => s.length != 0);
  }
  target = target.substring(0, target.length - 3);
  let new_id_list = [];
  for (id of current_id_list) {
    if (id != target) {
      new_id_list.push(id);
    }
  }
  if (source == "yt") {
    set_yt_id_list(new_id_list.join(","));
  } else {
    set_tw_id_list(new_id_list.join(","));
  }
  update_channel_id_list();
  update_video_list();
}
