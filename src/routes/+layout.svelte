<script>
  import '@material/web/all.js'
  import 'material-symbols'
  import '../app.css'
  import Appbar from '../components/Appbar.svelte'

  let { children } = $props()
</script>

<Appbar />

<main class="container">
  {@render children()}
</main>
