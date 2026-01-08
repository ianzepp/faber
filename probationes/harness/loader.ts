// Load tasks, examples, and config from YAML files

import { parse } from 'yaml'
import { join } from 'path'
import type { Task, Example, Model, TrialConfig } from './types'

const ROOT = join(import.meta.dir, '..')

export async function loadTasks(): Promise<Task[]> {
  const tasks: Task[] = []

  const taskFiles = [
    'tasks/translate.yml',
    'tasks/predict.yml',
    'tasks/complete.yml',
    'tasks/complex.yml',
    'tasks/control.yml',
  ]

  for (const file of taskFiles) {
    const content = await Bun.file(join(ROOT, file)).text()
    const parsed = parse(content)

    // Load file contents for file-based tasks
    for (const task of parsed.tasks) {
      if (task.input_faber_file) {
        task.input_faber = await Bun.file(join(ROOT, task.input_faber_file)).text()
      }
      if (task.expected_ts_file) {
        task.expected_ts = await Bun.file(join(ROOT, task.expected_ts_file)).text()
      }
      tasks.push(task)
    }
  }

  return tasks
}

export async function loadExamples(): Promise<Example[]> {
  const content = await Bun.file(join(ROOT, 'examples/base.yml')).text()
  const parsed = parse(content)
  return parsed.examples
}

export async function loadConfig(): Promise<{ models: Model[], trials: TrialConfig, framework_version: string }> {
  const content = await Bun.file(join(ROOT, 'config/models.yml')).text()
  const parsed = parse(content)
  return {
    models: parsed.models,
    trials: parsed.trials,
    framework_version: parsed.framework_version || '0.0',
  }
}
