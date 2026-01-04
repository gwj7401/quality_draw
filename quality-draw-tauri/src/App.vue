<script setup lang="ts">
import { ref, onMounted, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Department, DrawRecord, DrawResult } from './types'
import { getDepartmentTypeLabel, getSpecialtyTypeLabel, needsPressure, needsMechanical, formatDateTime } from './types'

// 页面状态
type PageName = 'draw' | 'history'
const currentPage = ref<PageName>('draw')

// 数据
const departments = ref<Department[]>([])
const records = ref<DrawRecord[]>([])

// 本轮已抽中状态
const currentRoundPressure = ref<[string, string][]>([])
const currentRoundMechanical = ref<[string, string][]>([])

// 抽签状态
const selectedDepartment = ref<Department | null>(null)
const isDrawing = ref(false)
const isWheelSpinning = ref(false)  // 转盘是否正在旋转
const canStop = ref(false)  // 是否可以停止
const drawPhase = ref<'select' | 'drawing' | 'result'>('select')
const pressureResult = ref<DrawResult | null>(null)
const mechanicalResult = ref<DrawResult | null>(null)
const pressureCandidates = ref<string[]>([])
const mechanicalCandidates = ref<string[]>([])
const wheelRotation = ref(0)
const wheelRotation2 = ref(0)

// 转盘动画相关
let animationId1: number | null = null
let animationId2: number | null = null
let currentSpeed1 = 0
let currentSpeed2 = 0
const maxSpeed = 25  // 增加最大速度
const acceleration = 1.2  // 加快加速度
const friction = 0.985  // 惯性摩擦系数（越接近1惯性越大）

// 当前显示的候选名称索引
const currentPressureIndex = ref(0)
const currentMechanicalIndex = ref(0)
let nameRollInterval: ReturnType<typeof setInterval> | null = null

// 加载数据
async function loadData() {
  try {
    departments.value = await invoke<Department[]>('get_departments')
    records.value = await invoke<DrawRecord[]>('get_records')
    const roundStatus = await invoke<[[string, string][], [string, string][]]>('get_current_round_status')
    currentRoundPressure.value = roundStatus[0]
    currentRoundMechanical.value = roundStatus[1]
  } catch (e) {
    console.error('Failed to load data:', e)
  }
}

onMounted(loadData)

onUnmounted(() => {
  if (animationId1) cancelAnimationFrame(animationId1)
  if (animationId2) cancelAnimationFrame(animationId2)
  if (nameRollInterval) clearInterval(nameRollInterval)
})

// 选择部门
function selectDepartment(dept: Department) {
  selectedDepartment.value = dept
  pressureResult.value = null
  mechanicalResult.value = null
}

// 检查部门是否在本轮已抽过
function alreadyDrew(deptId: string, type: 'pressure' | 'mechanical'): boolean {
  const list = type === 'pressure' ? currentRoundPressure.value : currentRoundMechanical.value
  return list.some(([target, _]) => target === deptId)
}

// 检查综合类部门是否完全抽完
function fullyDone(deptId: string): boolean {
  return alreadyDrew(deptId, 'pressure') && alreadyDrew(deptId, 'mechanical')
}

// 转盘动画函数（使用惯性摩擦实现真实减速效果）
function animateWheel1() {
  if (isWheelSpinning.value) {
    // 加速阶段
    if (currentSpeed1 < maxSpeed) {
      currentSpeed1 = Math.min(currentSpeed1 + acceleration, maxSpeed)
    }
  } else {
    // 惯性减速：使用乘法摩擦模拟真实惯性
    currentSpeed1 *= friction
    // 速度低于阈值时停止
    if (currentSpeed1 < 0.1) {
      currentSpeed1 = 0
    }
  }
  
  wheelRotation.value += currentSpeed1
  
  if (currentSpeed1 > 0) {
    animationId1 = requestAnimationFrame(animateWheel1)
  } else {
    animationId1 = null
  }
}

function animateWheel2() {
  if (isWheelSpinning.value) {
    // 加速阶段
    if (currentSpeed2 < maxSpeed) {
      currentSpeed2 = Math.min(currentSpeed2 + acceleration, maxSpeed)
    }
  } else {
    // 惯性减速：使用乘法摩擦模拟真实惯性
    currentSpeed2 *= friction
    // 速度低于阈值时停止
    if (currentSpeed2 < 0.1) {
      currentSpeed2 = 0
    }
  }
  
  wheelRotation2.value += currentSpeed2
  
  if (currentSpeed2 > 0) {
    animationId2 = requestAnimationFrame(animateWheel2)
  } else {
    animationId2 = null
  }
}

// 开始转盘动画
function startWheelAnimation() {
  isWheelSpinning.value = true
  currentSpeed1 = 0
  currentSpeed2 = 0
  
  if (!animationId1) {
    animateWheel1()
  }
  if (!animationId2) {
    animateWheel2()
  }
  
  // 名称轮播
  nameRollInterval = setInterval(() => {
    if (pressureCandidates.value.length > 0) {
      currentPressureIndex.value = (currentPressureIndex.value + 1) % pressureCandidates.value.length
    }
    if (mechanicalCandidates.value.length > 0) {
      currentMechanicalIndex.value = (currentMechanicalIndex.value + 1) % mechanicalCandidates.value.length
    }
  }, 100)
  
  // 1秒后允许停止
  setTimeout(() => {
    canStop.value = true
  }, 1000)
}

// 停止转盘并执行抽签
async function stopWheelAndDraw() {
  if (!canStop.value) return
  
  isWheelSpinning.value = false
  canStop.value = false
  
  if (nameRollInterval) {
    clearInterval(nameRollInterval)
    nameRollInterval = null
  }
  
  // 等待转盘惯性减速停止（大约3秒）
  await new Promise(resolve => setTimeout(resolve, 3500))
  
  const dept = selectedDepartment.value!
  const needsP = needsPressure(dept.department_type)
  const needsM = needsMechanical(dept.department_type)
  const drewP = alreadyDrew(dept.id, 'pressure')
  const drewM = alreadyDrew(dept.id, 'mechanical')
  
  // 执行抽签
  if (needsP && !drewP && pressureCandidates.value.length > 0) {
    pressureResult.value = await invoke<DrawResult>('execute_draw', {
      targetDepartmentId: dept.id,
      specialtyType: 'Pressure',
    })
  }
  
  if (needsM && !drewM && mechanicalCandidates.value.length > 0) {
    mechanicalResult.value = await invoke<DrawResult>('execute_draw', {
      targetDepartmentId: dept.id,
      specialtyType: 'Mechanical',
    })
  }
  
  isDrawing.value = false
  drawPhase.value = 'result'
  
  // 刷新数据
  await loadData()
}

// 开始抽签 - 进入抽签页面
async function startDraw() {
  if (!selectedDepartment.value) return
  
  const dept = selectedDepartment.value
  
  // 检查是否已抽过
  const needsP = needsPressure(dept.department_type)
  const needsM = needsMechanical(dept.department_type)
  const drewP = alreadyDrew(dept.id, 'pressure')
  const drewM = alreadyDrew(dept.id, 'mechanical')
  
  if ((needsP && drewP) && (needsM && drewM)) {
    alert('该部门本轮已抽过，请点击"开始新一轮"重新开始')
    return
  }
  if (needsP && drewP && !needsM) {
    alert('该部门本轮已抽过承压类，请点击"开始新一轮"重新开始')
    return
  }
  if (needsM && drewM && !needsP) {
    alert('该部门本轮已抽过机电类，请点击"开始新一轮"重新开始')
    return
  }
  
  isDrawing.value = true
  drawPhase.value = 'drawing'
  pressureResult.value = null
  mechanicalResult.value = null
  wheelRotation.value = 0
  wheelRotation2.value = 0
  
  // 获取候选部门名单
  if (needsP && !drewP) {
    pressureCandidates.value = await invoke<string[]>('get_candidate_departments', {
      targetDepartmentId: dept.id,
      specialtyType: 'Pressure',
    })
  } else {
    pressureCandidates.value = []
  }
  
  if (needsM && !drewM) {
    mechanicalCandidates.value = await invoke<string[]>('get_candidate_departments', {
      targetDepartmentId: dept.id,
      specialtyType: 'Mechanical',
    })
  } else {
    mechanicalCandidates.value = []
  }
  
  // 检查是否有候选部门 - 任一需要的类型候选为空都应该报错
  const pressureEmpty = needsP && !drewP && pressureCandidates.value.length === 0
  const mechanicalEmpty = needsM && !drewM && mechanicalCandidates.value.length === 0
  
  if (pressureEmpty && mechanicalEmpty) {
    alert('没有符合条件的候选部门！承压类和机电类候选都为空。')
    isDrawing.value = false
    drawPhase.value = 'select'
    return
  }
  
  if (pressureEmpty && needsP && !needsM) {
    alert('没有符合条件的承压类候选部门！')
    isDrawing.value = false
    drawPhase.value = 'select'
    return
  }
  
  if (mechanicalEmpty && needsM && !needsP) {
    alert('没有符合条件的机电类候选部门！')
    isDrawing.value = false
    drawPhase.value = 'select'
    return
  }
  
  // 检查是否只剩一个候选，自动执行抽签
  const pressureOnlyOne = needsP && !drewP && pressureCandidates.value.length === 1
  const mechanicalOnlyOne = needsM && !drewM && mechanicalCandidates.value.length === 1
  
  // 如果都只剩一个候选（或不需要），直接自动抽签
  const canAutoDrawPressure = !needsP || drewP || pressureOnlyOne
  const canAutoDrawMechanical = !needsM || drewM || mechanicalOnlyOne
  
  if (canAutoDrawPressure && canAutoDrawMechanical && (pressureOnlyOne || mechanicalOnlyOne)) {
    // 构造提示信息
    let autoMsg = '只剩最后的候选部门，自动选中：\n'
    if (pressureOnlyOne) {
      autoMsg += `承压类：${pressureCandidates.value[0]}\n`
    }
    if (mechanicalOnlyOne) {
      autoMsg += `机电类：${mechanicalCandidates.value[0]}\n`
    }
    
    // 显示提示
    alert(autoMsg)
    
    // 直接执行抽签
    if (pressureOnlyOne) {
      pressureResult.value = await invoke<DrawResult>('execute_draw', {
        targetDepartmentId: dept.id,
        specialtyType: 'Pressure',
      })
    }
    
    if (mechanicalOnlyOne) {
      mechanicalResult.value = await invoke<DrawResult>('execute_draw', {
        targetDepartmentId: dept.id,
        specialtyType: 'Mechanical',
      })
    }
    
    isDrawing.value = false
    drawPhase.value = 'result'
    await loadData()
    return
  }
}

// 返回选择
function backToSelect() {
  drawPhase.value = 'select'
  selectedDepartment.value = null
  pressureResult.value = null
  mechanicalResult.value = null
  wheelRotation.value = 0
  wheelRotation2.value = 0
  isWheelSpinning.value = false
  canStop.value = false
  pressureCandidates.value = []
  mechanicalCandidates.value = []
}

// 开始新一轮
async function startNewRound() {
  await invoke('start_new_round')
  await loadData()
  backToSelect()
}

// 清空记录
async function handleClearRecords() {
  if (confirm('确定要清空所有抽签记录吗？此操作不可恢复。')) {
    await invoke('clear_records')
    records.value = []
  }
}

// 导出 Excel
async function handleExport() {
  try {
    const path = await invoke<string>('export_to_excel')
    alert(`导出成功！\n文件保存在：${path}`)
  } catch (e) {
    alert(`导出失败：${e}`)
  }
}

// 导出 PDF
async function handleExportPdf() {
  try {
    const path = await invoke<string>('export_to_pdf')
    alert(`PDF导出成功！\n文件保存在：${path}`)
  } catch (e) {
    alert(`PDF导出失败：${e}`)
  }
}

// 获取部门名称

const groupedDepartments = computed(() => {
  const groups = {
    comprehensive: departments.value.filter(d => d.department_type === 'Comprehensive'),
    pressure: departments.value.filter(d => d.department_type === 'Pressure'),
    mechanical: departments.value.filter(d => d.department_type === 'Mechanical')
  }
  return groups
})

// 倒序记录
const sortedRecords = computed(() => {
  return [...records.value].reverse()
})

// 本轮抽中数量
const currentRoundCount = computed(() => {
  return currentRoundPressure.value.length + currentRoundMechanical.value.length
})

// 转盘扇区颜色
const sectorColors = ['#ff6b6b', '#feca57', '#48dbfb', '#ff9ff3', '#54a0ff', '#5f27cd', '#00d2d3', '#1dd1a1', '#ee5a24', '#686de0']

// 计算SVG扇区路径（处理极端情况）
function getSectorPath(index: number, total: number, radius: number = 125) {
  // 只有1个候选时，绘制整个圆
  if (total === 1) {
    return `M${radius},0 A${radius},${radius} 0 1 1 ${radius},${radius * 2} A${radius},${radius} 0 1 1 ${radius},0 Z`
  }
  
  const angle = 360 / total
  const startAngle = index * angle - 90
  const endAngle = startAngle + angle
  const startRad = (startAngle * Math.PI) / 180
  const endRad = (endAngle * Math.PI) / 180
  
  const x1 = radius + radius * Math.cos(startRad)
  const y1 = radius + radius * Math.sin(startRad)
  const x2 = radius + radius * Math.cos(endRad)
  const y2 = radius + radius * Math.sin(endRad)
  
  // 对于2个候选（180度），或更大角度，需要 largeArc = 1
  const largeArc = angle > 180 ? 1 : 0
  
  return `M${radius},${radius} L${x1},${y1} A${radius},${radius} 0 ${largeArc} 1 ${x2},${y2} Z`
}

// 计算文字位置
function getTextPosition(index: number, total: number, radius: number = 125) {
  const angle = 360 / total
  const midAngle = index * angle + angle / 2 - 90
  const rad = (midAngle * Math.PI) / 180
  const textRadius = radius * 0.65
  
  return {
    x: radius + textRadius * Math.cos(rad),
    y: radius + textRadius * Math.sin(rad),
    rotation: midAngle + 90
  }
}
</script>

<template>
  <div class="app-container">
    <!-- 粒子背景 -->
    <div class="particles">
      <div v-for="i in 20" :key="i" class="particle" 
           :style="{ 
             left: Math.random() * 100 + '%', 
             animationDelay: Math.random() * 20 + 's',
             animationDuration: 15 + Math.random() * 10 + 's'
           }"></div>
    </div>
    
    <!-- 头部 -->
    <header class="app-header">
      <div class="app-title">
        <div class="logo">🎲</div>
        <h1>宁夏特检院质量监督检查抽签程序</h1>
      </div>
      
      <nav class="nav-tabs">
        <button class="nav-tab" :class="{ active: currentPage === 'draw' }" @click="currentPage = 'draw'">
          🎯 抽签
        </button>
        <button class="nav-tab" :class="{ active: currentPage === 'history' }" @click="currentPage = 'history'">
          📋 历史记录
        </button>
      </nav>
    </header>
    
    <!-- 主内容 -->
    <main class="app-main">
      <!-- 抽签页面 -->
      <div v-if="currentPage === 'draw'">
        <!-- 选择阶段 -->
        <div v-if="drawPhase === 'select'">
          <div class="card mb-24">
            <div class="card-title">
              <div class="icon">🏢</div>
              选择被检查部门
            </div>
            
            <!-- 综合类 -->
            <div v-if="groupedDepartments.comprehensive.length" class="mb-24">
              <h3 class="text-secondary mb-16">综合类部门（需要抽取承压类和机电类两个部门）</h3>
              <div class="department-grid">
                <div 
                  v-for="dept in groupedDepartments.comprehensive" 
                  :key="dept.id"
                  class="department-card"
                  :class="{ 
                    selected: selectedDepartment?.id === dept.id,
                    done: fullyDone(dept.id),
                    partial: alreadyDrew(dept.id, 'pressure') || alreadyDrew(dept.id, 'mechanical')
                  }"
                  @click="selectDepartment(dept)"
                >
                  <div class="name">
                    <span v-if="fullyDone(dept.id)">✓ </span>
                    <span v-else-if="alreadyDrew(dept.id, 'pressure') || alreadyDrew(dept.id, 'mechanical')">◐ </span>
                    {{ dept.name }}
                  </div>
                  <div class="type comprehensive">{{ getDepartmentTypeLabel(dept.department_type) }}</div>
                </div>
              </div>
            </div>
            
            <!-- 承压类 -->
            <div v-if="groupedDepartments.pressure.length" class="mb-24">
              <h3 class="text-secondary mb-16">承压类部门</h3>
              <div class="department-grid">
                <div 
                  v-for="dept in groupedDepartments.pressure" 
                  :key="dept.id"
                  class="department-card"
                  :class="{ 
                    selected: selectedDepartment?.id === dept.id,
                    done: alreadyDrew(dept.id, 'pressure')
                  }"
                  @click="selectDepartment(dept)"
                >
                  <div class="name">
                    <span v-if="alreadyDrew(dept.id, 'pressure')">✓ </span>
                    {{ dept.name }}
                  </div>
                  <div class="type pressure">{{ getDepartmentTypeLabel(dept.department_type) }}</div>
                </div>
              </div>
            </div>
            
            <!-- 机电类 -->
            <div v-if="groupedDepartments.mechanical.length">
              <h3 class="text-secondary mb-16">机电类部门</h3>
              <div class="department-grid">
                <div 
                  v-for="dept in groupedDepartments.mechanical" 
                  :key="dept.id"
                  class="department-card"
                  :class="{ 
                    selected: selectedDepartment?.id === dept.id,
                    done: alreadyDrew(dept.id, 'mechanical')
                  }"
                  @click="selectDepartment(dept)"
                >
                  <div class="name">
                    <span v-if="alreadyDrew(dept.id, 'mechanical')">✓ </span>
                    {{ dept.name }}
                  </div>
                  <div class="type mechanical">{{ getDepartmentTypeLabel(dept.department_type) }}</div>
                </div>
              </div>
            </div>
          </div>
          
          <!-- 控制区域 -->
          <div class="flex items-center justify-center gap-24 flex-wrap">
            <button 
              class="btn btn-primary btn-lg" 
              :disabled="!selectedDepartment"
              @click="startDraw"
            >
              🎲 开始抽签
            </button>
            
            <button class="btn btn-outline" @click="startNewRound">
              🔄 开始新一轮
            </button>
            
            <div v-if="currentRoundCount > 0" class="text-secondary">
              本轮已抽: {{ currentRoundCount }}
            </div>
          </div>
          
          <p v-if="selectedDepartment" class="text-center text-secondary mt-16">
            已选择：{{ selectedDepartment.name }}
            <span v-if="selectedDepartment.department_type === 'Comprehensive'">
              （将抽取承压类和机电类各一个部门）
            </span>
          </p>
        </div>


        <!-- 抽签动画阶段 -->
        <div v-if="drawPhase === 'drawing'" class="draw-stage">
          <h2 class="text-xl font-bold text-center mb-8" :class="{ 'animate-pulse': isWheelSpinning }">
            {{ isWheelSpinning ? '正在抽签...' : (canStop ? '准备开始' : '抽签准备就绪') }}
          </h2>
          <p class="text-secondary text-center mb-16">被检部门：{{ selectedDepartment?.name }}</p>
          
          <!-- 当前轮播显示的候选名称 -->
          <div v-if="isWheelSpinning" class="current-name-display">
            <div v-if="pressureCandidates.length > 0" class="name-box pressure-name">
              <span class="label">承压类：</span>
              <span class="name">{{ pressureCandidates[currentPressureIndex] }}</span>
            </div>
            <div v-if="mechanicalCandidates.length > 0" class="name-box mechanical-name">
              <span class="label">机电类：</span>
              <span class="name">{{ mechanicalCandidates[currentMechanicalIndex] }}</span>
            </div>
          </div>
          
          <!-- 转盘区域 -->
          <div class="wheel-area">
            <div class="wheel-container">
            <!-- 承压类转盘 -->
            <div v-if="pressureCandidates.length > 0" class="wheel-wrapper">
              <div class="wheel-outer-ring">
                <div v-for="i in 12" :key="i" class="bulb" :class="{ 'bulb-active': isWheelSpinning }" :style="{ transform: `rotate(${i * 30}deg) translateY(-155px)` }"></div>
              </div>
              <div class="wheel-border">
                <svg class="wheel-svg" :style="{ transform: `rotate(${wheelRotation}deg)` }" viewBox="0 0 250 250">
                  <!-- 扇区 -->
                  <g v-for="(name, index) in pressureCandidates" :key="index">
                    <path 
                      :d="getSectorPath(index, pressureCandidates.length)" 
                      :fill="sectorColors[index % sectorColors.length]"
                      stroke="#333"
                      stroke-width="1"
                    />
                    <text 
                      :x="getTextPosition(index, pressureCandidates.length).x"
                      :y="getTextPosition(index, pressureCandidates.length).y"
                      :transform="`rotate(${getTextPosition(index, pressureCandidates.length).rotation}, ${getTextPosition(index, pressureCandidates.length).x}, ${getTextPosition(index, pressureCandidates.length).y})`"
                      text-anchor="middle"
                      dominant-baseline="middle"
                      fill="#fff"
                      font-size="11"
                      font-weight="bold"
                      style="text-shadow: 1px 1px 2px rgba(0,0,0,0.8);"
                    >{{ name.length > 5 ? name.slice(0, 5) + '..' : name }}</text>
                  </g>
                </svg>
              </div>
              <div class="wheel-pointer"></div>
              <div class="wheel-center-decoration">
                <div class="inner-text">承压类<br>部门</div>
              </div>
              <div class="wheel-label mt-16 font-bold text-primary text-center">承压类检查部门</div>
            </div>
            
            <!-- 机电类转盘 -->
            <div v-if="mechanicalCandidates.length > 0" class="wheel-wrapper">
              <div class="wheel-outer-ring">
                <div v-for="i in 12" :key="i" class="bulb" :class="{ 'bulb-active': isWheelSpinning }" :style="{ transform: `rotate(${i * 30}deg) translateY(-155px)` }"></div>
              </div>
              <div class="wheel-border">
                <svg class="wheel-svg" :style="{ transform: `rotate(${wheelRotation2}deg)` }" viewBox="0 0 250 250">
                  <!-- 扇区 -->
                  <g v-for="(name, index) in mechanicalCandidates" :key="index">
                    <path 
                      :d="getSectorPath(index, mechanicalCandidates.length)" 
                      :fill="sectorColors[index % sectorColors.length]"
                      stroke="#333"
                      stroke-width="1"
                    />
                    <text 
                      :x="getTextPosition(index, mechanicalCandidates.length).x"
                      :y="getTextPosition(index, mechanicalCandidates.length).y"
                      :transform="`rotate(${getTextPosition(index, mechanicalCandidates.length).rotation}, ${getTextPosition(index, mechanicalCandidates.length).x}, ${getTextPosition(index, mechanicalCandidates.length).y})`"
                      text-anchor="middle"
                      dominant-baseline="middle"
                      fill="#fff"
                      font-size="11"
                      font-weight="bold"
                      style="text-shadow: 1px 1px 2px rgba(0,0,0,0.8);"
                    >{{ name.length > 5 ? name.slice(0, 5) + '..' : name }}</text>
                  </g>
                </svg>
              </div>
              <div class="wheel-pointer"></div>
              <div class="wheel-center-decoration">
                 <div class="inner-text">机电类<br>部门</div>
              </div>
              <div class="wheel-label mt-16 font-bold text-success text-center">机电类检查部门</div>
            </div>
            </div>
          </div>
          
          <!-- 控制按钮 -->
          <div class="control-buttons">
            <button 
              v-if="!isWheelSpinning" 
              class="btn btn-primary btn-lg start-btn"
              @click="startWheelAnimation"
            >
              ▶ 开始转动
            </button>
            <button 
              v-else 
              class="btn btn-danger btn-lg stop-btn"
              :disabled="!canStop"
              @click="stopWheelAndDraw"
            >
              {{ canStop ? '⏹ 停止抽签' : '请稍候...' }}
            </button>
            <button class="btn btn-outline" @click="backToSelect">
              ← 返回选择
            </button>
          </div>
        </div>


        <!-- 结果阶段 -->
        <div v-if="drawPhase === 'result'" class="draw-stage">
          <h2 class="text-xl font-bold mb-24 text-center">🎉 抽签结果</h2>
          <p class="text-secondary mb-24 text-center">被检部门：{{ selectedDepartment?.name }}</p>
          
          <div class="flex gap-32 flex-wrap justify-center">
            <!-- 承压类结果 -->
            <div v-if="pressureResult" class="result-card" :class="{ success: pressureResult.success }">
              <div v-if="pressureResult.success">
                <div class="result-icon">🛡️</div>
                <div class="result-type">承压类检查部门</div>
                <div class="result-name">{{ pressureResult.department_name }}</div>
              </div>
              <div v-else class="text-secondary p-16">
                {{ pressureResult.message || '无符合条件的承压类候选部门' }}
              </div>
            </div>
            
            <!-- 机电类结果 -->
            <div v-if="mechanicalResult" class="result-card" :class="{ success: mechanicalResult.success }">
              <div v-if="mechanicalResult.success">
                <div class="result-icon">⚡</div>
                <div class="result-type">机电类检查部门</div>
                <div class="result-name">{{ mechanicalResult.department_name }}</div>
              </div>
              <div v-else class="text-secondary p-16">
                {{ mechanicalResult.message || '无符合条件的机电类候选部门' }}
              </div>
            </div>
          </div>
          
          <div class="mt-32 flex gap-16 justify-center">
            <button class="btn btn-primary" @click="backToSelect">继续抽签</button>
            <button class="btn btn-outline" @click="currentPage = 'history'">查看记录</button>
          </div>
        </div>
      </div>
      
      <!-- 历史记录页面 -->
      <div v-if="currentPage === 'history'">
        <div class="card">
          <div class="flex justify-between items-center mb-24">
            <div class="card-title" style="margin-bottom: 0;">
              <div class="icon">📋</div>
              抽签历史记录
            </div>
            <div class="flex gap-16">
              <button class="btn btn-success" @click="handleExport" :disabled="records.length === 0">
                📊 导出 Excel
              </button>
              <button class="btn btn-primary" @click="handleExportPdf" :disabled="records.length === 0">
                📄 导出 PDF
              </button>
              <button class="btn btn-danger" @click="handleClearRecords" :disabled="records.length === 0">
                🗑️ 清空记录
              </button>
            </div>
          </div>
          
          <div v-if="records.length === 0" class="text-center text-secondary" style="padding: 48px;">
            暂无抽签记录
          </div>
          
          <div v-else class="table-container">
            <table class="table">
              <thead>
                <tr>
                  <th>序号</th>
                  <th>抽签时间</th>
                  <th>被检部门</th>
                  <th>专责类型</th>
                  <th>抽中部门</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(record, index) in sortedRecords" :key="record.id">
                  <td>{{ records.length - index }}</td>
                  <td>{{ formatDateTime(record.timestamp) }}</td>
                  <td>{{ record.target_department_name }}</td>
                  <td>
                    <span class="badge" :class="record.specialty_type === 'Pressure' ? 'badge-primary' : 'badge-success'">
                      {{ getSpecialtyTypeLabel(record.specialty_type) }}
                    </span>
                  </td>
                  <td><strong>{{ record.selected_specialist_name }}</strong></td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.department-card.done {
  border-color: var(--success-color);
  background: rgba(16, 185, 129, 0.1);
}

.department-card.partial {
  border-color: var(--warning-color);
  background: rgba(245, 158, 11, 0.1);
}

.department-card.done .name,
.department-card.partial .name {
  color: var(--text-secondary);
}

/* 转盘相关样式 */
.wheel-container {
  display: flex;
  justify-content: center;
  gap: 60px;
  padding: 20px;
}

.wheel-wrapper {
  position: relative;
  width: 320px;
  height: 320px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

/* 外围灯泡圈 */
.wheel-outer-ring {
  position: absolute;
  top: 50%;
  left: 50%;
  width: 290px;
  height: 290px;
  transform: translate(-50%, -50%);
  border-radius: 50%;
  pointer-events: none;
}

.bulb {
  position: absolute;
  top: 50%;
  left: 50%;
  width: 8px;
  height: 8px;
  background: #444;
  border-radius: 50%;
  box-shadow: 0 0 2px #000;
  animation: bulb-blink 1s infinite alternate;
}

.wheel-wrapper:nth-child(2) .bulb {
  animation-delay: 0.5s;
}

@keyframes bulb-blink {
  from { background: #444; box-shadow: 0 0 2px #000; }
  to { background: #ffd700; box-shadow: 0 0 10px #ffd700; }
}

/* 金属边框 */
.wheel-border {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 270px;
  height: 270px;
  border-radius: 50%;
  background: linear-gradient(135deg, #444 0%, #222 50%, #111 100%);
  box-shadow: 
    0 0 0 8px #222,
    0 0 0 10px #444,
    0 0 20px rgba(0,0,0,0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1;
}

.wheel {
  width: 250px;
  height: 250px;
  border-radius: 50%;
  position: relative;
  transition: transform 3s cubic-bezier(0.1, 0.8, 0.1, 1);
  /* 使用 conic-gradient 创建多彩扇区 */
  background: conic-gradient(
    #ff6b6b 0% 12.5%,
    #feca57 12.5% 25%,
    #48dbfb 25% 37.5%,
    #ff9ff3 37.5% 50%,
    #54a0ff 50% 62.5%,
    #5f27cd 62.5% 75%,
    #c8d6e5 75% 87.5%,
    #1dd1a1 87.5% 100%
  );
  box-shadow: inset 0 0 20px rgba(0,0,0,0.5);
  border: 4px solid #333;
}

.wheel::after {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 50%;
  background: radial-gradient(transparent 50%, rgba(0,0,0,0.3) 100%);
  pointer-events: none;
}

/* 中心装饰 */
.wheel-center-decoration {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 80px;
  height: 80px;
  border-radius: 50%;
  background: linear-gradient(135deg, #fff 0%, #e0e0e0 100%);
  border: 4px solid #d4d4d4;
  box-shadow: 
    0 4px 10px rgba(0,0,0,0.3),
    inset 0 0 10px rgba(255,255,255,0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
}

.wheel-center-decoration .inner-text {
  font-size: 14px;
  font-weight: bold;
  color: #333;
  text-align: center;
  line-height: 1.2;
}

/* 指针 */
.wheel-pointer {
  position: absolute;
  top: 10px; /* 调整位置到圆盘上方 */
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  height: 0;
  border-left: 15px solid transparent;
  border-right: 15px solid transparent;
  border-top: 30px solid #ff4757;
  filter: drop-shadow(0 4px 4px rgba(0,0,0,0.5));
  z-index: 20;
}

/* 结果卡片样式 */
.result-card {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  padding: 30px;
  min-width: 250px;
  text-align: center;
  backdrop-filter: blur(10px);
  transition: all 0.3s ease;
}

.result-card.success {
  border-color: var(--primary-color);
  background: radial-gradient(circle at center, rgba(59, 130, 246, 0.15) 0%, rgba(255,255,255,0.05) 100%);
  box-shadow: 0 10px 30px rgba(59, 130, 246, 0.2);
}

.result-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.result-type {
  color: var(--text-secondary);
  font-size: 14px;
  margin-bottom: 8px;
}

.result-name {
  font-size: 24px;
  font-weight: bold;
  color: #fff;
}

/* SVG转盘样式 */
.wheel-svg {
  width: 250px;
  height: 250px;
  border-radius: 50%;
  box-shadow: inset 0 0 20px rgba(0,0,0,0.5);
}

/* 灯泡激活状态 */
.bulb-active {
  animation: bulb-blink-fast 0.2s infinite alternate !important;
}

@keyframes bulb-blink-fast {
  from { background: #ffd700; box-shadow: 0 0 5px #ffd700; }
  to { background: #ff6b6b; box-shadow: 0 0 15px #ff6b6b; }
}

/* 当前名称显示区域 */
.current-name-display {
  display: flex;
  justify-content: center;
  gap: 40px;
  margin-bottom: 24px;
  flex-wrap: wrap;
}

.name-box {
  background: rgba(0, 0, 0, 0.6);
  border: 2px solid;
  border-radius: 12px;
  padding: 12px 24px;
  min-width: 200px;
  text-align: center;
  animation: name-pulse 0.3s ease-in-out infinite;
}

.name-box .label {
  opacity: 0.7;
  font-size: 14px;
}

.name-box .name {
  font-size: 22px;
  font-weight: bold;
  display: block;
  margin-top: 4px;
}

.pressure-name {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.mechanical-name {
  border-color: var(--success-color);
  color: var(--success-color);
}

@keyframes name-pulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.02); }
}

/* 控制按钮区域 */
.control-buttons {
  display: flex;
  justify-content: center;
  gap: 20px;
  flex-wrap: wrap;
}

.start-btn {
  animation: pulse-glow 1.5s ease-in-out infinite;
}

@keyframes pulse-glow {
  0%, 100% { 
    box-shadow: 0 0 10px var(--primary-color);
  }
  50% { 
    box-shadow: 0 0 30px var(--primary-color), 0 0 50px var(--primary-color);
  }
}

.stop-btn {
  animation: danger-pulse 0.8s ease-in-out infinite;
}

@keyframes danger-pulse {
  0%, 100% { 
    background: linear-gradient(135deg, #ff4757 0%, #c0392b 100%);
  }
  50% { 
    background: linear-gradient(135deg, #ff6b6b 0%, #e74c3c 100%);
  }
}

.stop-btn:disabled {
  animation: none;
  opacity: 0.6;
  cursor: not-allowed;
}
</style>