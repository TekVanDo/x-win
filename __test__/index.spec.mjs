import test from 'ava';
import os from 'os';
import { activeWindow, openWindows, subscribeActiveWindow, unsubscribeActiveWindow, unsubscribeAllActiveWindow } from '../index.js';

const defaultStruct = {
  os: os.platform(),
  info: { execName: "", name: "", path: "", processId: 0 },
  position: { height: 0, width: 0, x: 0, y: 0 },
  processId: 0,
  title: "",
  usage: { memory: 0 },
};

/**
 * Compare struct 
 * @param {*} t 
 * @param {*} data 
 */
function compareStruct(t, data) {
  const defaultkeys = Object.entries(defaultStruct);
  for (const [key, value] of defaultkeys) {
    /** For darwin with permission issue should ignore title it will be empty */
    if (os.platform() === 'darwin' && key === 'title') {
      continue;
    }
    if (!(key === 'title' && data.os === 'win32' && data.info.execName === 'Widgets')) {
      if (key === 'os') {
        t.deepEqual(value, data[key]);
      } else {
        t.notDeepEqual(value, data[key]);
      }
    }
  }
}

test('activeWindow', (t) => {
  console.time('activeWindow');
  const data = activeWindow();
  console.timeEnd('activeWindow');
  t.log(data);
  compareStruct(t, data);
})

test('openWindows', (t) => {
  console.time('openwindows');
  const list = openWindows();
  console.timeEnd('openwindows');
  t.log(list);
  for (const data of list) {
    compareStruct(t, data);
  }
})

test('subscribeActiveWindow', async (t) => {
  const data1 = await new Promise((resolve, reject) => {
    console.time('subscribeActiveWindow1');
    const r = subscribeActiveWindow((info) => {
      console.timeEnd('subscribeActiveWindow1');
      t.log(r, info);
      resolve(info);
      unsubscribeActiveWindow(r);
    });
  });

  const data2 = await new Promise((resolve, reject) => {
    console.time('subscribeActiveWindow2');
    const r = subscribeActiveWindow((info) => {
      console.timeEnd('subscribeActiveWindow2');
      t.log(r, info);
      resolve(info);
      unsubscribeActiveWindow(r);
    });
  });

  const data3 = await new Promise((resolve, reject) => {
    console.time('subscribeActiveWindow3');
    const r = subscribeActiveWindow((info) => {
      console.timeEnd('subscribeActiveWindow3');
      t.log(r, info);
      resolve(info);
      unsubscribeActiveWindow(r);
    });
  });
  compareStruct(t, data1);
  compareStruct(t, data2);
  compareStruct(t, data3);
})


test('unsubscribeAllActiveWindow', async (t) => {
  const data1 = await new Promise((resolve, reject) => {
    const r = subscribeActiveWindow((info) => {
      t.log(r, info);
      resolve(info);
    });
  });

  const data2 = await new Promise((resolve, reject) => {
    const r = subscribeActiveWindow((info) => {
      t.log(r, info);
      resolve(info);
    });
  });

  const data3 = await new Promise((resolve, reject) => {
    const r = subscribeActiveWindow((info) => {
      t.log(r, info);
      resolve(info);
    });
  });
  compareStruct(t, data1);
  compareStruct(t, data2);
  compareStruct(t, data3);
  unsubscribeAllActiveWindow();
})
