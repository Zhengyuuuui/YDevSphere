import type { Project, Workspace } from './types'

export const WORKSPACES: Workspace[] = [
  { id: 'desktop', name: 'Desktop', path: '~/Desktop' },
  { id: 'documents', name: 'Documents', path: '~/Documents' },
  { id: 'developer', name: 'Custom Workspace', path: '~/Developer' },
]

export const PROJECTS: Project[] = [
  {
    id: '1',
    name: 'client',
    path: '~/Desktop/client',
    technologies: [
      { name: 'Node.js', variant: 'node' },
      { name: 'Vue', variant: 'vue' },
    ],
    git: { type: 'clean' },
    updatedAt: '12m',
    lastOpenedAt: '12m',
  },
  {
    id: '2',
    name: 'ydevsphere',
    path: '~/Developer/ydevsphere',
    technologies: [
      { name: 'Rust', variant: 'rust' },
      { name: 'Vue', variant: 'vue' },
    ],
    git: { type: 'clean' },
    updatedAt: '1h',
    lastOpenedAt: '2h',
  },
  {
    id: '3',
    name: 'page-flip',
    path: '~/Desktop/page-flip',
    technologies: [{ name: 'Node.js', variant: 'node' }],
    git: { type: 'dirty', changes: 3 },
    updatedAt: '2d',
    lastOpenedAt: '3d',
  },
  {
    id: '4',
    name: 'figma-make-app',
    path: '~/Documents/figma-make-app',
    technologies: [
      { name: 'React', variant: 'react' },
      { name: 'TypeScript', variant: 'typescript' },
    ],
    git: { type: 'clean' },
    updatedAt: '3d',
    lastOpenedAt: '3d',
  },
  {
    id: '5',
    name: 'vibe-diary-server',
    path: '~/Desktop/vibe-diary-server',
    technologies: [
      { name: 'Node.js', variant: 'node' },
      { name: 'TypeScript', variant: 'typescript' },
    ],
    git: { type: 'clean' },
    updatedAt: '3d',
    lastOpenedAt: '3d',
  },
  {
    id: '6',
    name: 'canglan-flash-backend',
    path: '~/Developer/canglan-flash-backend',
    technologies: [
      { name: 'Java', variant: 'java' },
      { name: 'Spring', variant: 'spring' },
    ],
    git: { type: 'dirty', changes: 2 },
    updatedAt: '5d',
    lastOpenedAt: '1w',
  },
  {
    id: '7',
    name: 'canglan-flash-frontend',
    path: '~/Developer/canglan-flash-frontend',
    technologies: [
      { name: 'Vue', variant: 'vue' },
      { name: 'TypeScript', variant: 'typescript' },
    ],
    git: { type: 'clean' },
    updatedAt: '1w',
    lastOpenedAt: '1w',
  },
  {
    id: '8',
    name: 'campus-wall-backend',
    path: '~/Developer/campus-wall-backend',
    technologies: [
      { name: 'Node.js', variant: 'node' },
      { name: 'TypeScript', variant: 'typescript' },
    ],
    git: { type: 'clean' },
    updatedAt: '2w',
    lastOpenedAt: '2w',
  },
  {
    id: '9',
    name: 'campus-wall-uniapp',
    path: '~/Developer/campus-wall-uniapp',
    technologies: [{ name: 'Vue', variant: 'vue' }],
    git: { type: 'dirty', changes: 1 },
    updatedAt: '2w',
    lastOpenedAt: '3w',
  },
  {
    id: '10',
    name: 'banking-backend',
    path: '~/Developer/banking-backend',
    technologies: [{ name: 'Java', variant: 'java' }],
    git: { type: 'none' },
    updatedAt: '3w',
    lastOpenedAt: '1mo',
  },
]
