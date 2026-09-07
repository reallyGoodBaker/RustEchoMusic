/**
 * RustEchoMusic 插件宿主 SDK
 *
 * 注入到插件 iframe 内，封装 postMessage 协议为 Promise-based API。
 * 零依赖 IIFE，挂载到 window.rem。
 * 协议信封：{ source, type, id?, payload? }
 */
(function (window) {
  'use strict';

  var HOST_SOURCE = 'rem-plugin-host';
  var PLUGIN_SOURCE = 'rem-plugin';
  var DEFAULT_TIMEOUT = 10000; // 请求默认超时 10s

  // 请求-响应配对表：id -> { resolve, reject, timer }
  var pendingRequests = new Map();
  // 事件监听器：eventName -> Set<Function>
  var listeners = new Map();
  // 握手信息（收到宿主 ready 后填充）
  var readyInfo = null;
  var readyResolvers = [];

  // 生成唯一请求 id，优先用 crypto.randomUUID，降级到时间戳+随机串
  function genId() {
    if (window.crypto && typeof window.crypto.randomUUID === 'function') {
      return window.crypto.randomUUID();
    }
    return 'r_' + Date.now().toString(36) + '_' + Math.random().toString(36).slice(2, 10);
  }

  // 向宿主发送消息信封
  function send(type, payload, id) {
    var envelope = { source: PLUGIN_SOURCE, type: type };
    if (id !== undefined) envelope.id = id;
    if (payload !== undefined) envelope.payload = payload;
    window.parent.postMessage(envelope, '*');
  }

  // 发起请求并等待对应 result
  function request(type, payload) {
    return new Promise(function (resolve, reject) {
      var id = genId();
      var timer = window.setTimeout(function () {
        if (pendingRequests.has(id)) {
          pendingRequests.delete(id);
          reject(new Error('请求超时: ' + type + ' (' + id + ')'));
        }
      }, DEFAULT_TIMEOUT);
      pendingRequests.set(id, { resolve: resolve, reject: reject, timer: timer });
      send(type, payload, id);
    });
  }

  // 按 kind 派发事件给监听器，单个异常不影响其他
  function dispatch(kind, data) {
    var set = listeners.get(kind);
    if (!set) return;
    set.forEach(function (cb) {
      try { cb(data); } catch (e) { /* swallow */ }
    });
  }

  // 处理请求响应：command 始终 { ok, data|error }；其余兼容裸值与错误信封
  function handleResult(data) {
    var pending = pendingRequests.get(data.id);
    if (!pending) return;
    window.clearTimeout(pending.timer);
    pendingRequests.delete(data.id);
    var payload = data.payload;

    if (data.type === 'command:result') {
      if (payload && payload.ok === false) {
        pending.reject(new Error(payload.error || '命令执行失败'));
      } else {
        pending.resolve(payload ? payload.data : payload);
      }
      return;
    }
    // settings:get / settings:set / state:get：裸值原样返回，错误信封 reject
    if (payload && typeof payload === 'object' && payload.ok === false) {
      pending.reject(new Error(payload.error || ('请求失败: ' + data.type)));
      return;
    }
    pending.resolve(payload);
  }

  // 处理宿主 → 插件消息
  function onMessage(event) {
    var data = event.data;
    if (!data || data.source !== HOST_SOURCE) return;

    switch (data.type) {
      case 'ready':
        readyInfo = data.payload || {};
        readyResolvers.forEach(function (r) { r.resolve(readyInfo); });
        readyResolvers = [];
        dispatch('ready', readyInfo);
        break;
      case 'event': {
        var p = data.payload || {};
        dispatch(p.kind, p.data);
        break;
      }
      case 'command:result':
      case 'settings:get:result':
      case 'settings:set:result':
      case 'state:get:result':
        handleResult(data);
        break;
      default:
        break; // 未知类型忽略
    }
  }

  window.addEventListener('message', onMessage);

  var sdk = {
    // 等待宿主握手，返回 { pluginId, capabilities }
    ready: function () {
      return new Promise(function (resolve) {
        if (readyInfo) {
          resolve(readyInfo);
        } else {
          readyResolvers.push({ resolve: resolve });
        }
      });
    },

    // 执行宿主命令：command 名 + args
    command: function (command, args) {
      return request('command', { command: command, args: args });
    },

    settings: {
      get: function (key) {
        return request('settings:get', { key: key });
      },
      set: function (key, value) {
        return request('settings:set', { key: key, value: value });
      }
    },

    state: {
      get: function (kind) {
        return request('state:get', { kind: kind });
      }
    },

    // 订阅事件：ready | trackChanged | playbackStateChanged | queueChanged | settingsChanged
    on: function (eventName, callback) {
      if (!listeners.has(eventName)) listeners.set(eventName, new Set());
      listeners.get(eventName).add(callback);
      if (eventName !== 'ready' && listeners.get(eventName).size === 1) {
        // 首次监听非 ready 事件时通知宿主
        send('subscribe', { kind: eventName });
      } else if (eventName === 'ready' && readyInfo) {
        // 已握手完成则异步补发，避免错过 ready
        Promise.resolve().then(function () { callback(readyInfo); });
      }
      return sdk;
    },

    off: function (eventName, callback) {
      var set = listeners.get(eventName);
      if (set) {
        set.delete(callback);
        if (set.size === 0 && eventName !== 'ready') {
          listeners.delete(eventName);
          send('unsubscribe', { kind: eventName });
        }
      }
      return sdk;
    }
  };

  window.rem = sdk;
})(window);
