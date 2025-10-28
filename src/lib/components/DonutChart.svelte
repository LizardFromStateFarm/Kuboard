<!-- Kuboard Donut Chart Component -->
<script lang="ts">
  export let value: number = 0; // 0-100 percentage
  export let label: string = '';
  export let color: string = '#10b981'; // Default green
  export let size: number = 120; // Size in pixels
  export let strokeWidth: number = 8; // Stroke width
  export let showPercentage: boolean = true;
  export let showLabel: boolean = true;
  export let animated: boolean = true;

  // Calculate the circumference and offset for the donut
  $: radius = (size - strokeWidth) / 2;
  $: circumference = 2 * Math.PI * radius;
  $: offset = circumference - (value / 100) * circumference;

  // Animation delay based on value for staggered effect
  $: animationDelay = value * 10; // ms delay
</script>

<div class="donut-chart-container" style="width: {size}px; height: {size}px;">
  <svg 
    class="donut-chart" 
    width={size} 
    height={size}
    viewBox={`0 0 ${size} ${size}`}
  >
    <!-- Background circle -->
    <circle
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="none"
      stroke="rgba(255, 255, 255, 0.1)"
      stroke-width={strokeWidth}
      class="background-circle"
    />
    
    <!-- Progress circle -->
    <circle
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="none"
      stroke={color}
      stroke-width={strokeWidth}
      stroke-linecap="round"
      stroke-dasharray={circumference}
      stroke-dashoffset={circumference}
      class="progress-circle"
      class:animated={animated}
      style={animated ? `animation-delay: ${animationDelay}ms;` : `stroke-dashoffset: ${offset};`}
    />
  </svg>
  
  <!-- Center content -->
  <div class="donut-center">
    {#if showPercentage}
      <div class="percentage" style="color: {color};">
        {value.toFixed(1)}%
      </div>
    {/if}
    {#if showLabel}
      <div class="label">{label}</div>
    {/if}
  </div>
</div>

<style>
  .donut-chart-container {
    position: relative;
    display: inline-block;
  }

  .donut-chart {
    transform: rotate(-90deg); /* Start from top */
  }

  .background-circle {
    opacity: 0.2;
  }

  .progress-circle {
    transition: stroke-dashoffset 0.3s ease-in-out;
  }

  .progress-circle.animated {
    animation: draw-circle 1.5s ease-in-out forwards;
  }

  @keyframes draw-circle {
    from {
      stroke-dashoffset: var(--circumference);
    }
    to {
      stroke-dashoffset: var(--offset);
    }
  }

  .donut-center {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    text-align: center;
    pointer-events: none;
  }

  .percentage {
    font-size: 1.5rem;
    font-weight: 700;
    line-height: 1;
    margin-bottom: 0.25rem;
  }

  .label {
    font-size: 0.75rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.8);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* Color variants with improved contrast */
  .progress-circle[stroke="#10b981"] {
    filter: drop-shadow(0 0 8px var(--shadow-success));
  }

  .progress-circle[stroke="#3b82f6"] {
    filter: drop-shadow(0 0 8px var(--shadow-primary));
  }

  .progress-circle[stroke="#f59e0b"] {
    filter: drop-shadow(0 0 8px var(--shadow-warning));
  }

  .progress-circle[stroke="#ef4444"] {
    filter: drop-shadow(0 0 8px var(--shadow-error));
  }

  .progress-circle[stroke="#8b5cf6"] {
    filter: drop-shadow(0 0 8px rgba(139, 92, 246, 0.4));
  }

  .progress-circle[stroke="#06b6d4"] {
    filter: drop-shadow(0 0 8px var(--shadow-info));
  }

  /* Responsive sizing */
  @media (max-width: 768px) {
    .percentage {
      font-size: 1.25rem;
    }
    
    .label {
      font-size: 0.7rem;
    }
  }
</style>
