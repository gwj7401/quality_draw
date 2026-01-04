// 类型定义 - 按部门抽签版本

export type DepartmentType = 'Comprehensive' | 'Pressure' | 'Mechanical'
export type SpecialtyType = 'Pressure' | 'Mechanical'

export interface Department {
    id: string
    name: string
    department_type: DepartmentType
}

export interface DrawRecord {
    id: string
    timestamp: string
    target_department_id: string
    target_department_name: string
    specialty_type: SpecialtyType
    selected_specialist_id: string      // 实际是部门ID
    selected_specialist_name: string    // 实际是部门名称
    selected_from_department_id: string
    selected_from_department_name: string
}

export interface DrawResult {
    success: boolean
    department_name?: string
    department_id?: string
    specialty_type?: string
    message?: string
}

// 工具函数
export function getDepartmentTypeLabel(type: DepartmentType): string {
    const labels: Record<DepartmentType, string> = {
        Comprehensive: '综合类',
        Pressure: '承压类',
        Mechanical: '机电类'
    }
    return labels[type] || type
}

export function getSpecialtyTypeLabel(type: SpecialtyType): string {
    const labels: Record<SpecialtyType, string> = {
        Pressure: '承压类',
        Mechanical: '机电类'
    }
    return labels[type] || type
}

export function needsPressure(type: DepartmentType): boolean {
    return type === 'Comprehensive' || type === 'Pressure'
}

export function needsMechanical(type: DepartmentType): boolean {
    return type === 'Comprehensive' || type === 'Mechanical'
}

export function formatDateTime(isoString: string): string {
    try {
        const date = new Date(isoString)
        return date.toLocaleString('zh-CN', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit'
        })
    } catch {
        return isoString
    }
}
