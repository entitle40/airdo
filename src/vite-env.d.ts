/// <reference types="vite/client" />
/// <reference types="vite-svg-loader" />
declare module '*.vue' {
    import {ComponentOptions} from 'vue'
    const componentOptions: ComponentOptions
    export default componentOptions
}
