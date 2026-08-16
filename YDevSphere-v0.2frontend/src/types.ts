export type TechVariant =
  | 'node'
  | 'vue'
  | 'react'
  | 'rust'
  | 'typescript'
  | 'python'
  | 'java'
  | 'spring'
  | 'nextjs'
  | 'neutral'

export interface Technology {
  name: string
  variant: TechVariant
}

export type GitStatusType = 'clean' | 'dirty' | 'none' | 'detached'

export interface GitStatus {
  type: GitStatusType
  changes?: number
}

export interface Project {
  id: string
  name: string
  path: string
  technologies: Technology[]
  git: GitStatus
  updatedAt: string
  lastOpenedAt?: string
}

export type Page = 'overview' | 'projects' | 'recent' | 'settings'

export interface Workspace {
  id: string
  name: string
  path: string
}
