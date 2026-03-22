import { createContext, useContext, type ParentProps } from "solid-js"

export function createSimpleContext<T>(name: string) {
  const Context = createContext<T>()

  const useValue = () => {
    const value = useContext(Context)
    if (!value) {
      throw new Error(`${name} context is missing`)
    }
    return value
  }

  const Provider = (props: ParentProps<{ value: T }>) => {
    return <Context.Provider value={props.value}>{props.children}</Context.Provider>
  }

  return [Provider, useValue] as const
}
