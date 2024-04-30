<script setup lang="ts">
import pc from '../assets/pc.json';
import * as echarts from 'echarts';
import proxy_api, {HealthCheck} from '../api/proxy'
import {onMounted, ref, nextTick} from "vue";
import {ElMessage} from "element-plus";

interface CascaderOption {
  value: string,
  label: string,
  children?: CascaderOption[]
}

const pcValue = ref()
let pcOption: CascaderOption[] = [];
const pcType = pc as {[key: string]: string[]}
for (let pcKey in pcType) {
  let children = []
  for (const child of pcType[pcKey]) {
    let item = {
      value: child,
      label: child,
    }
    children.push(item)
  }
  let item = {
    value: pcKey,
    label: pcKey,
    children: children,
  }
  pcOption.push(item)
}

const broadband = ref()
const broadbandOption = ["移动", "联通", "电信"]
const broadbandEnv = ref()
const broadbandEnvOption = ["家宽", "服务器"]
const isUsing = ref()
const isUsingOption = ["测试中*有*使用测试机器或测试带宽看视频、玩游戏、浏览网页等行为", "测试中*没有*使用测试机器或测试带宽看视频、玩游戏、浏览网页等行为"]

const endTime = new Date()
const startTime = new Date(endTime.getTime() - 60 * 60 * 1000)
const datetimeRange = ref([startTime, endTime])
const datetimeRangeShortcuts = [
  {
    text: 'Last 5 minutes',
    value: () => {
      const end = new Date()
      const start = new Date(endTime.getTime() - 5 * 60 * 1000)
      return [start, end]
    },
  },
  {
    text: 'Last 15 minutes',
    value: () => {
      const end = new Date()
      const start = new Date(endTime.getTime() - 15 * 60 * 1000)
      return [start, end]
    },
  },
  {
    text: 'Last 30 minutes',
    value: () => {
      const end = new Date()
      const start = new Date(endTime.getTime() - 35 * 60 * 1000)
      return [start, end]
    },
  },
  {
    text: 'Last 1 hour',
    value: () => {
      const end = new Date()
      const start = new Date(endTime.getTime() - 60 * 60 * 1000)
      return [start, end]
    },
  },
  {
    text: 'Last 3 hour',
    value: () => {
      const end = new Date()
      const start = new Date(endTime.getTime() - 3 * 60 * 60 * 1000)
      return [start, end]
    },
  },
  {
    text: 'Last 6 hour',
    value: () => {
      const end = new Date()
      const start = new Date(endTime.getTime() - 6 * 60 * 60 * 1000)
      return [start, end]
    },
  },
  {
    text: 'Last 12 hour',
    value: () => {
      const end = new Date()
      const start = new Date(endTime.getTime() - 12 * 60 * 60 * 1000)
      return [start, end]
    },
  },
  {
    text: 'Last 24 hour',
    value: () => {
      const end = new Date()
      const start = new Date(endTime.getTime() - 24 * 60 * 60 * 1000)
      return [start, end]
    },
  },
]

onMounted(() => {
  refreshHealthCheckList()
})

const loading = ref(false)
const list = ref<HealthCheck[]>([])
function refreshHealthCheckList() {
  loading.value = true
  proxy_api.list_health_check({
    start_time: Math.floor(datetimeRange.value[0].getTime() / 1000),
    end_time: Math.floor(datetimeRange.value[1].getTime() / 1000)
  }).then(({data}) => {
    if (data.code != 200) {
      ElMessage.error({
        message: "获取健康检查列表错误：" + data.message
      })
      return
    }
    list.value = data.data
    refreshCharts()
  }).catch(e => {
    console.error(e)
    ElMessage.error({
      message: "获取健康检查列表错误：" + e
    })
  })
}

const chartsName = ref<string[]>([])
function refreshCharts() {
  let map: {[key: string]: [number, number][]} = {};
  for (let i = 0; i < list.value.length; i++) {
    let item = list.value[i]
    let delay_list = map[item.node_name]
    if (!delay_list) {
      delay_list = []
      map[item.node_name] = delay_list
    }
    delay_list.push([list.value[i].request_time * 1000, list.value[i].delay_ms])
  }
  chartsName.value = Object.keys(map).sort()
  for (let mapKey in map) {
    let option: echarts.EChartsOption = {
      title: {
        left: 'center',
        text: mapKey
      },
      xAxis: {
        type: 'time'
      },
      yAxis: {
        type: 'value'
      },
      tooltip: {
        show: true,
        trigger: 'axis',
        axisPointer: {
          type: 'line',
        },
        showContent: true,
      },
      dataZoom: [
        {
          id: 'dataZoomX',
          type: 'slider',
          xAxisIndex: [0],
          filterMode: 'filter'
        },
      ],
      series: [
        {
          name: mapKey,
          type: 'line',
          smooth: true,
          symbol: 'none',
          areaStyle: {},
          data: map[mapKey]
        }
      ]
    };
    nextTick(() => {
      const chart = echarts.init(document.getElementById('chart-' + mapKey));
      chart.setOption(option);
      loading.value = false
    })
  }
}
</script>

<template>
  <div>
    <div>
      <el-cascader
          v-model="pcValue"
          :options="pcOption"
          filterable
          clearable
          placeholder="省市"
          style="width: 330px"
          :props="{expandTrigger: 'hover' as const}"
      />
      <el-select
          v-model="broadband"
          placeholder="宽带"
          filterable
          allow-create
          clearable
          style="width: 80px"
      >
        <el-option
            v-for="item in broadbandOption"
            :key="item"
            :label="item"
            :value="item"
        />
      </el-select>
      <el-select
          v-model="broadbandEnv"
          placeholder="宽带环境"
          filterable
          allow-create
          clearable
          style="width: 120px"
      >
        <el-option
            v-for="item in broadbandEnvOption"
            :key="item"
            :label="item"
            :value="item"
        />
      </el-select>
      <el-select
          v-model="isUsing"
          placeholder="测试中是否使用测试机器或测试带宽看视频、玩游戏、浏览网页等行为"
          filterable
          allow-create
          clearable
          style="width: 460px"
      >
        <el-option
            v-for="item in isUsingOption"
            :key="item"
            :label="item"
            :value="item"
        />
      </el-select>
      <el-input type="textarea" autosize placeholder="备注，在这里说点什么" />
    </div>
    <div>
      <el-date-picker
          v-model="datetimeRange"
          type="datetimerange"
          :shortcuts="datetimeRangeShortcuts"
          @change="refreshHealthCheckList"
          range-separator="To"
          start-placeholder="开始日期和时间"
          end-placeholder="结束日期和时间"
      />
    </div>
    <div v-for="name of chartsName" :id="'chart-' + name" :key="'chart-' + name"  v-loading="loading" style="width: 100%; height: 50vh"></div>
    <div>
      * 作者声明：本次测试仅代表特定时间段、特定宽带环境、特定机场下的延迟情况，实际情况请以自行测试为准。
    </div>
  </div>
</template>

<style scoped>
</style>
