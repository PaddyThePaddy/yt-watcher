const DAY_MS = 1000 * 60 * 60 * 24;
const HOUR_MS = 1000 * 60 * 60;
const MIN_MS = 1000 * 60;

let site_url = document.URL;
let channel_data = null;
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

function get_id_list() {
  const cookies = document.cookie.split(";");
  for (c of cookies) {
    const pair = c.split("=", 2);
    if (pair[0].trim() == "id_list") {
      return pair[1].trim();
    }
  }
  return "";
}

function set_id_list(id_list) {
  document.cookie = "id_list=" + id_list;
}

function load_channel(value) {
  return fetch(
    site_url + "channel?q=" + value
  )
    .then((resp) => {
      return resp.json();
    })
    .then((resp) => {
      if (resp.error == null) {
        channel_data = resp.data;
      }
    });
}

function build_channel_div(title, url, thumbnail_url) {
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

function check_channel() {
  load_channel(document.getElementById("channel_name").value)    .then(() => {
    
    if (channel_data != null) {
      let preview = build_channel_div(
        channel_data.title,
        channel_data.custom_url,
        channel_data.thumbnail
      );
      let preview_frame = document.getElementById("channel_preview");
      while (preview_frame.childElementCount != 0) {
        preview_frame.removeChild(preview_frame.firstChild);
      }
      preview_frame.appendChild(preview);
      document.getElementById("follow_btn").hidden = false;
      document.getElementById("check_btn").hidden = true;
    } else {
      document.getElementById("follow_btn").hidden = true;
      document.getElementById("check_btn").hidden = false;
      clear_id();
    }
  });
}

function import_channel_list() {
  let list = document.getElementById("channel_name").value;
  list = list.replace(new RegExp(site_url + "\\w+\\?channels=([^&]+)"), "$1")
  let promises = [];
  for (c of list.split(",")) {
    promises.push(load_channel(c).then(() => {
      follow_channel();
    }));
  }
  Promise.all(promises).then(() => {update_video_list()});
}

function follow_btn() {
  follow_channel()
  update_video_list();
}

function follow_channel() {
  if (!channel_data) {
    console.log("url is not valid");
    return;
  }
  let exist = false;
  let id_list = get_id_list();
  for (id of id_list.split(",")) {
    if (!id.startsWith("@")) {
      id = "@" + id;
    }
    if (id == channel_data.custom_url) {
      exist = true;
      break;
    }
  }
  if (!exist) {
    if (id_list.length != 0) {
      id_list += ",";
    }
    let url = channel_data.custom_url;
    if (url.startsWith("@")) {
      url = url.substring(1)
    }
    id_list += url;
    set_id_list(id_list);
    update_channel_id_list();
  }
}

function clear_id() {
  let preview_frame = document.getElementById("channel_preview");
  while (preview_frame.childElementCount != 0) {
    preview_frame.removeChild(preview_frame.firstChild);
  }
  channel_data = null;
  document.getElementById("follow_btn").hidden = true;
  document.getElementById("check_btn").hidden = false;
}

function update_video_list() {
  const id_list = get_id_list();
  console.log((new Date()) + " updating video info")
  if (id_list.length == 0) {
    console.log("id list is empty");
    return;
  }
  fetch(site_url + "data?channels=" + id_list)
    .then((resp) => {
      return resp.json();
    })
    .then((data) => (video_data = data))
    .then(() => {
      render_video_list();
    });
}

function render_video_list() {
  const id_list = get_id_list();
  console.log((new Date()) + " rendering video list")
  if (id_list.length == 0) {
    console.log("id list is empty");
    return;
  }

  const ongoing_video_frame = document.getElementById("ongoing_video_frame");
  const upcoming_video_frame = document.getElementById("upcoming_video_frame");
  while (ongoing_video_frame.childElementCount != 0) {
    ongoing_video_frame.removeChild(ongoing_video_frame.firstChild);
  }
  while (upcoming_video_frame.childElementCount != 0) {
    upcoming_video_frame.removeChild(upcoming_video_frame.firstChild);
  }
  for (data of video_data) {
    if (data.ongoing) {
      ongoing_video_frame.appendChild(build_video_preview(data));
    } else {
      upcoming_video_frame.appendChild(build_video_preview(data));
    }
  }
}

function build_video_preview(data) {
  const frame = document.createElement("div");
  const link = document.createElement("a");
  link.href = data.target_url;
  const img = document.createElement("img");
  img.src = data.thumbnail_url;
  const title = document.createElement("span");
  title.innerHTML = data.title;
  title.classList = "video_title";
  const start_time = new Date(data.start_timestamp_millis);
  const time_span = document.createElement("span");
  time_span.innerHTML = get_time_delta_string(start_time);
  time_span.title = start_time.toLocaleString();
  link.appendChild(img);
  link.appendChild(document.createElement("br"));
  link.appendChild(title);
  link.appendChild(document.createElement("br"));
  link.appendChild(time_span);
  frame.appendChild(link);
  frame.classList = "video_frame";
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
  navigator.clipboard
    .writeText(site_url + "cal?channels=" + get_id_list())
    .then(() => {
      console.log("copy success");
      document.getElementById("copy_popup").classList.add("show")
      if (copy_timeout_handle) {
        clearTimeout(copy_timeout_handle);
      }
      copy_timeout_handle = setTimeout(() => {
        document.getElementById("copy_popup").classList.remove("show")
        copy_timeout_handle = null;
      }, 2000);
    });
}
function open_menu() {
  const menu = document.getElementById("menu_content");
  menu.style.width = "var(--menu-width)";
  menu.style.padding = "5px";
  menu.style.overflow = "scroll";
  document.getElementById("menu_remaining_area").style.visibility = "visible"
}

function close_menu() {
  const menu = document.getElementById("menu_content");
  menu.style.padding = "0px";
  menu.style.overflow = "hidden";
  menu.style.width = "0px";
  document.getElementById("menu_remaining_area").style.visibility = "hidden"
}

function update_channel_id_list() {
  const channel_id_list = get_id_list().split(",");
  const id_list_parent = document.getElementById("channel_id_list");
  while (id_list_parent.childElementCount != 0) {
    id_list_parent.removeChild(id_list_parent.firstChild);
  }
  for (id of channel_id_list) {
    if (id.trim().length > 0) {
      id_list_parent.appendChild(build_channel_id_item(id));
    }
  }
}

function build_channel_id_item(id) {
  const item = document.createElement("li");
  const id_span = document.createElement("span");
  id_span.innerHTML = "@" + id;
  const del_btn = document.createElement("button");
  del_btn.innerHTML = "&times";
  del_btn.onclick = () => {
    del_channel(id);
  };
  item.appendChild(del_btn);
  item.appendChild(id_span);
  return item;
}

function del_channel(target) {
  let current_id_list = get_id_list().split(",");
  let new_id_list = [];
  for (id of current_id_list) {
    if (id != target) {
      new_id_list.push(id);
    }
  }
  set_id_list(new_id_list.join(","));
  update_channel_id_list();
}
