<script>
  let { value, classList } = $props()
  import { onMount } from 'svelte'

  let sliderRef = $state(null)

  const size = 1.5

  const styleText = `
          .handle .elevation,
          .handle::before {
            position: absolute;
            display: block;
            content: " ";
            left: ${size / 4}rem!important;
            top: ${size / 4}rem!important;
            width: ${size / 2}rem!important;
            height: ${size / 2}rem!important;
          }

          .handle {
            width: ${size}rem!important;
            height: ${size}rem!important;
            margin-top: ${-size / 2}rem!important;
          }
          `

  function updateSliderStyle(elem) {
    const shadow = elem?.shadowRoot
    if (shadow) {
      const childNodes = Array.from(shadow.childNodes)

      childNodes.forEach(childNode => {
        if (childNode.nodeName === 'STYLE') {
          console.log(childNode.textContent)
          childNode.textContent += styleText
        }
      })

      if (!childNodes.some(node => node.nodeName === 'STYLE')) {
        const style = document.createElement('style')
        style.textContent = styleText
        shadow.appendChild(style)
      }
    }
  }

  onMount(() => {
    updateSliderStyle(sliderRef)
  })
</script>

<mdui-slider nolabel value class={classList} bind:this={sliderRef}
></mdui-slider>
